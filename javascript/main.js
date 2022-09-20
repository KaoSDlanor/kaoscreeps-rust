const kaoScreeps = {
  wasm : null,

  log : (...args) => {
    console.log(...args);
    Game.notify(args.join(' '));
  },

  loop : () => {
    try {
      global.Memory = { spawns : {}, creeps : {} };
      if (kaoScreeps.wasm == null) {
        kaoScreeps.wasm = require('kaoscreeps-rust');
        kaoScreeps.wasm.initialize_instance();
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