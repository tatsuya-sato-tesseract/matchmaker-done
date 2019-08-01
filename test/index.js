const path = require("path");
const tape = require("tape");

const {
  Diorama,
  tapeExecutor,
  backwardCompatibilityMiddleware
} = require("@holochain/diorama");

process.on("unhandledRejection", error => {
  // Will print "unhandledRejection err is not defined"
  console.error("got unhandledRejection:", error);
});

const dnaPath = path.join(__dirname, "../dist/matchmaker-tats.dna.json");
const dna = Diorama.dna(dnaPath, "matchmaker-tats");

const diorama = new Diorama({
  instances: {
    alice: dna,
    bob: dna
  },
  bridges: [],
  debugLog: false,
  executor: tapeExecutor(require("tape")),
  middleware: backwardCompatibilityMiddleware
});

diorama.registerScenario(
  "Can create a new game",
  async (s, t, { alice, bob }) => {
    const create_game_result = await alice.callSync("main", "create_game", {
      opponent: bob.agentId,
      timestamp: 0
    });
    t.equal(create_game_result.Ok.length, 46);

    const make_move_1_result = await bob.callSync("main", "make_move", {
      new_move: {
        game: create_game_result.Ok,
        move_type: { Suggest: { suggestion: 5 } },
        timestamp: 1
      }
    });
    t.equal(make_move_1_result.Err, undefined);

    const make_move_2_result = await alice.callSync("main", "make_move", {
      new_move: {
        game: create_game_result.Ok,
        move_type: { Predict: { prediction: 5 } },
        timestamp: 2
      }
    });
    t.equal(make_move_2_result.Err, undefined);

    const render_state_result = await alice.callSync("main", "render_state", {
      game_address: create_game_result.Ok
    });
    console.log(render_state_result.Ok);
    t.equal(render_state_result.Err, undefined);
  }
);

diorama.run();
