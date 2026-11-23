//! RNG seed probe: production formulas and migration evidence only.
//!
//! This file intentionally does not change production code. It prints concrete values for
//! the current TurnFixed/fork/reset/rollout paths, a historical XOR model, DECISION_TAG,
//! and CRN sharing. It makes no collision guarantee.

use std::collections::BTreeSet;
use std::sync::Mutex;

use anyhow::Result;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use umaai_core::Data;
use umaai_core::crn::RolloutSeeds;
use umaai_core::rng::{
    DECISION_TAG, EVENT_TAG, GAMMA, STRATEGY_TAG, StreamTag, TURN_FIXED_TAG, TurnFixedRng, derive_seed, splitmix64,
};
use umaai_core::testing::data_root;
use umaai_tests::testing::{RaceFirstTrainer, basic_newgame};
use umasim::game::Game;
use umasim::game::base::BasicStage;
use umasim::search::{FlatSearch, FlatSearchGame};

fn historical_xor(base: u64, parts: &[u64]) -> u64 {
    splitmix64(parts.iter().fold(base, |acc, part| acc ^ part))
}

fn hex(v: u64) -> String {
    format!("0x{v:016x}")
}

fn distribution_for(master: u64, turn: i32, turn_fixed_override: Option<u64>) -> Result<(Vec<Vec<i32>>, u64, u64)> {
    let mut game = basic_newgame()?;
    game.turn = turn;
    game.stage = BasicStage::Distribute;
    game.set_rule_master(master);
    let original_strategy = game.strategy.as_ref().map(|rng| rng.master()).unwrap_or(0);
    let mut streams = game.base.take_rule_streams()?;
    let original_event = streams.event.master();
    if let Some(candidate) = turn_fixed_override {
        streams.turn_fixed = TurnFixedRng::new(candidate);
    }
    game.base.put_rule_streams(streams);
    game.run_stage(&RaceFirstTrainer, &mut StdRng::seed_from_u64(0))?;
    let streams = game.base.take_rule_streams()?;
    let event_master = streams.event.master();
    game.base.put_rule_streams(streams);
    assert_eq!(event_master, original_event, "仅替换 TurnFixed 不得改变事件流");
    let strategy_master = game.strategy.as_ref().map(|rng| rng.master()).unwrap_or(0);
    assert_eq!(strategy_master, original_strategy, "仅替换 TurnFixed 不得改变策略流");
    Ok((game.distribution.clone(), event_master, strategy_master))
}

#[test]
fn rng_seed_probe_production_paths_and_formulae() -> Result<()> {
    Data::init(data_root()?)?;

    let master = 42u64;
    let turn = 0u64;
    let rollout_seed = 0x0123_4567_89ab_cdefu64;

    let current_turn_fixed = derive_seed(master, &[turn]);
    let tagged_turn_fixed = derive_seed(master, &[turn, TURN_FIXED_TAG]);
    let historical_turn_fixed = historical_xor(master, &[turn]);
    let current_event = derive_seed(master, &[turn, EVENT_TAG]);
    let current_strategy = derive_seed(master, &[turn, STRATEGY_TAG]);

    let mut game = basic_newgame()?;
    game.set_rule_master(master);
    let streams = game.base.take_rule_streams()?;
    let observed_turn_fixed = streams.turn_fixed.master();
    let observed_event = streams.event.master();
    game.base.put_rule_streams(streams);

    println!("production master={master} turn={turn}");
    println!(
        "TURN_FIXED_TAG={} DECISION_TAG={} EVENT_TAG={} STRATEGY_TAG={}",
        hex(TURN_FIXED_TAG),
        hex(DECISION_TAG),
        hex(EVENT_TAG),
        hex(STRATEGY_TAG)
    );
    println!(
        "turn_fixed current={} tagged={} historical_xor={}",
        hex(current_turn_fixed),
        hex(tagged_turn_fixed),
        hex(historical_turn_fixed)
    );
    println!(
        "event current={} strategy current={}",
        hex(current_event),
        hex(current_strategy)
    );
    println!(
        "observed BasicGame streams: turn_fixed={} event={}",
        hex(observed_turn_fixed),
        hex(observed_event)
    );

    assert_eq!(
        observed_turn_fixed, tagged_turn_fixed,
        "生产 TurnFixed 现在应为 derive_seed(master, [turn, TURN_FIXED_TAG])"
    );
    assert_eq!(
        observed_event, current_event,
        "生产 Event 当前公式应为 derive_seed(master, [turn, EVENT_TAG])"
    );
    assert_ne!(
        observed_turn_fixed, current_turn_fixed,
        "带标签后必须与旧的裸回合派生区分开（迁移即改值）"
    );
    assert_ne!(
        observed_turn_fixed, historical_turn_fixed,
        "当前生产值与历史 XOR 对照应可区分"
    );

    // 回合迁移范围样本：确认「裸回合」与「回合 + 标签」在每个回合都不同值。
    for sampled_turn in [0u64, 1, 10, 77] {
        let current = derive_seed(master, &[sampled_turn]);
        let tagged = derive_seed(master, &[sampled_turn, TURN_FIXED_TAG]);
        println!(
            "turn={sampled_turn} TurnFixed current={} tagged={} changed={}",
            hex(current),
            hex(tagged),
            current != tagged
        );
        assert_ne!(
            current, tagged,
            "显式标签迁移在回合 {sampled_turn} 改变 TurnFixed master"
        );
    }

    let mut reset_stream = TurnFixedRng::new(1);
    let _ = reset_stream.next_u64();
    reset_stream.reset(current_turn_fixed);
    assert_eq!(reset_stream.counter(), 0, "TurnFixed reset 必须归零计数");
    assert_eq!(
        reset_stream.next_u64(),
        TurnFixedRng::new(current_turn_fixed).next_u64()
    );

    let mut fork = game.fork_for_rollout(rollout_seed);
    assert_eq!(fork.rule_master(), Some(rollout_seed));
    let fork_streams = fork.base.take_rule_streams()?;
    let fork_turn_fixed = fork_streams.turn_fixed.master();
    let fork_event = fork_streams.event.master();
    fork.base.put_rule_streams(fork_streams);
    println!(
        "fork_for_rollout seed={} -> turn_fixed={} event={}",
        hex(rollout_seed),
        hex(fork_turn_fixed),
        hex(fork_event)
    );
    assert_eq!(
        fork_turn_fixed,
        derive_seed(rollout_seed, &[turn, TURN_FIXED_TAG])
    );
    assert_eq!(fork_event, derive_seed(rollout_seed, &[turn, EVENT_TAG]));

    let mut reset_game = game.clone();
    reset_game.set_rule_master(rollout_seed);
    let reset_streams = reset_game.base.take_rule_streams()?;
    assert_eq!(
        reset_streams.turn_fixed.master(),
        fork_turn_fixed,
        "set_rule_master/reset 路径与 fork_for_rollout 必须同值"
    );
    assert_eq!(
        reset_streams.event.master(),
        fork_event,
        "set_rule_master/reset 事件流与 fork_for_rollout 必须同值"
    );
    reset_game.base.put_rule_streams(reset_streams);

    Ok(())
}

#[test]
fn rng_seed_probe_decision_tag_and_crn_samples() -> Result<()> {
    Data::init(data_root()?)?;

    let entry_seed = 7u64;
    let mut entry = StdRng::seed_from_u64(entry_seed);
    let root = entry.next_u64();
    let seeds = RolloutSeeds::from_root(root);
    let sample: Vec<u64> = (0..8).map(|j| seeds.seed_at(j)).collect();

    let mut crn_unique = BTreeSet::new();
    for &seed in &sample {
        crn_unique.insert(seed);
    }
    println!(
        "CRN entry_seed={} root={} seeds[0..8]={:?}",
        entry_seed,
        hex(root),
        sample.iter().map(|x| hex(*x)).collect::<Vec<_>>()
    );
    assert_eq!(crn_unique.len(), sample.len(), "样本内 rollout seed 应各不相同");

    // 真实 FlatSearch 生产调用：闭包按候选收集实际传入的 seed 序列。
    let game = basic_newgame()?;
    let actions = game.list_actions()?;
    let seen = Mutex::new(Vec::<(usize, u64)>::new());
    let cfg = umasim::search::SearchConfig::default().with_ucb(false).with_search_n(4);
    let search = FlatSearch::new(cfg).with_parallel(false);
    let mut search_rng = StdRng::seed_from_u64(entry_seed);
    search.search_with(&game, &actions, &mut search_rng, |_, action, seed| {
        let idx = actions
            .iter()
            .position(|candidate| candidate == action)
            .expect("候选应来自 actions");
        seen.lock().expect("seed capture lock").push((idx, seed));
        Ok(0.0)
    })?;
    let captured = seen.into_inner().expect("seed capture lock");
    let expected_sample = &sample[..4];
    for idx in 0..actions.len() {
        let got: Vec<u64> = captured
            .iter()
            .filter(|(i, _)| *i == idx)
            .map(|(_, seed)| *seed)
            .collect();
        assert_eq!(
            got, expected_sample,
            "候选 {idx} 的实际 seed_at 序列未与 CRN 表前 4 项一致"
        );
    }
    println!(
        "FlatSearch actual candidates={} captured_calls={} each_rollouts=4",
        actions.len(),
        captured.len()
    );
    assert_eq!(search_rng, entry, "搜索只从入口决策流取一个根种子");
    assert_ne!(
        root,
        derive_seed(entry_seed, &[DECISION_TAG]),
        "实际搜索根种子来自入口 RNG next_u64，不是 DECISION_TAG 派生别名"
    );
    println!(
        "DECISION_TAG={} actual_root={} derive(entry_seed,[DECISION_TAG])={} relation=none",
        hex(DECISION_TAG),
        hex(root),
        hex(derive_seed(entry_seed, &[DECISION_TAG]))
    );

    // 顺序、零分量：历史 XOR 对照确实不敏感；当前逐段加法敏感。
    assert_eq!(historical_xor(9, &[1, 2]), historical_xor(9, &[2, 1]));
    assert_eq!(historical_xor(42, &[7]), historical_xor(42, &[7, 0]));
    assert_ne!(derive_seed(9, &[1, 2]), derive_seed(9, &[2, 1]));
    assert_ne!(derive_seed(42, &[7]), derive_seed(42, &[7, 0]));

    // 多 master/turn 实跑：迁移前的「裸回合」口径与历史 XOR 都会改变真实分配。
    let mut bare_changes = 0usize;
    let mut legacy_changes = 0usize;
    for master in [0u64, 42, 43, 0x0123_4567_89ab_cdef] {
        for turn in [0i32, 1, 2, 10] {
            let (current, current_event, current_strategy) = distribution_for(master, turn, None)?;
            let bare_master = derive_seed(master, &[turn as u64]);
            let legacy_master = historical_xor(master, &[turn as u64]);
            let (bare, bare_event, bare_strategy) = distribution_for(master, turn, Some(bare_master))?;
            let (legacy, legacy_event, legacy_strategy) =
                distribution_for(master, turn, Some(legacy_master))?;
            assert_eq!(bare_event, current_event, "bare TurnFixed 候选不得改变事件流");
            assert_eq!(bare_strategy, current_strategy, "bare TurnFixed 候选不得改变策略流");
            assert_eq!(legacy_event, current_event, "legacy TurnFixed 候选不得改变事件流");
            assert_eq!(legacy_strategy, current_strategy, "legacy TurnFixed 候选不得改变策略流");
            // 生产现在已带标签，所以「Bare」就是迁移前的旧口径
            let bare_changed = current != bare;
            let legacy_changed = current != legacy;
            bare_changes += usize::from(bare_changed);
            legacy_changes += usize::from(legacy_changed);
            println!(
                "distribution master={} turn={} bare_changed={} legacy_changed={}",
                master, turn, bare_changed, legacy_changed
            );
        }
    }
    println!(
        "distribution change counts bare={} legacy={} samples=16",
        bare_changes, legacy_changes
    );
    assert!(bare_changes > 0, "迁移前的裸回合口径至少应改变一个真实分配样本");
    assert!(legacy_changes > 0, "历史 XOR 至少应改变一个真实分配样本");

    // 明确反例：历史 XOR 模型会把 (base, part) 与 (base^k, part^k) 合并。
    let k = 0x55u64;
    let a = historical_xor(0x1000, &[0x2200]);
    let b = historical_xor(0x1000 ^ k, &[0x2200 ^ k]);
    println!(
        "historical XOR collision witness: a={} b={} equal={}",
        hex(a),
        hex(b),
        a == b
    );
    assert_eq!(a, b, "该反例只验证历史 XOR 对照模型的已知碰撞关系");
    assert_ne!(
        derive_seed(0x1000, &[0x2200]),
        derive_seed(0x1000 ^ k, &[0x2200 ^ k]),
        "当前逐段加法 derive_seed 在该反例上不复现 XOR 等值"
    );
    assert_eq!(GAMMA, 0x9E37_79B9_7F4A_7C15);
    assert_eq!(StreamTag::TurnFixed.tag(), TURN_FIXED_TAG);
    Ok(())
}
