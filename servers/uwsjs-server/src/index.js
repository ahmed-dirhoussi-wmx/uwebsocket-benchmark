const { App, DISABLED } = require("uWebSockets.js");

const enc = new TextDecoder("utf-8");

const WRITE_FACTOR = 2;

App()
  .ws("/*", {
    /* There are many common helper features */
    idleTimeout: 0,
    maxBackpressure: 256 * 1024,
    sendPingsAutomatically: false,
    closeOnBackpressureLimit: false,
    compression: DISABLED,

    message: async (ws, message, isBinary) => {
      let json_message = JSON.parse(enc.decode(message));

      // Recv to send factor 4x.
      const now = Date.now();
      // console.log(`Server latency : ${now - json_message.created_at}.`);
      for (let index = 0; index < WRITE_FACTOR; index++) {
        const server_msg = {
          client_id: json_message.client_id,
          msg_id: json_message.client_id,
          msg: json_message.msg,
          created_at: now,
          client_ts: json_message.created_at,
          server_latency: Math.round(now - json_message.created_at),
        };
        let ok = ws.send(JSON.stringify(server_msg), isBinary, false);
        // console.log(`${ok}`)
      }
    },
  })
  .listen(3000, (listenSocket) => {
    if (listenSocket) {
      console.log("Listening to port 3000");
    }
  });
