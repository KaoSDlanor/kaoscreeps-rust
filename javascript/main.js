const kaoScreeps = {
  wasm : null,

  log : (...args) => {
    console.log(...args);
    Game.notify(args.join(' '));
  },

  loop : () => {
    console.log(`GAME TICK ${Game.time}`)
    try {
      global.Memory = { spawns : {}, creeps : {} };
      if (kaoScreeps.wasm == null) {
        kaoScreeps.log('Loading code');
        kaoScreeps.wasm = require('kaoscreeps-rust');
        kaoScreeps.wasm.initialize_instance();
        kaoScreeps.wasm.setup();
      }
      kaoScreeps.wasm.game_loop();
    } catch(err) {
      kaoScreeps.log('Panic!', err);
      if (err.stack) {
        kaoScreeps.log('TRACE:', err.stack);
      }
      kaoScreeps.log('resetting VM next tick.');
      kaoScreeps.wasm = null;
    }
  },
};

module.exports = kaoScreeps;