use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(
        "file:output/jex-sc-stablepool.wasm",
        jex_sc_stablepool::ContractBuilder,
    );

    blockchain
}

#[test]
fn empty_rs() {
    world().run("scenarios/estimate_add_liquidity_underlying.scen.json");
}
