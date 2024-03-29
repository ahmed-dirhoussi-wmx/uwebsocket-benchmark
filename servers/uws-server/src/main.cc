#include <chrono>

#include <App.h>
#include <nlohmann/json.hpp>

constexpr int WRITE_FACTOR = 4;

int main() {
  struct PerSocketData {};

  uWS::App()
    .ws<PerSocketData>("/*", {.compression = uWS::CompressOptions(uWS::DISABLED),
                              .idleTimeout = 0,
                              .maxBackpressure = 256 * 1024,
                              .closeOnBackpressureLimit = false,
                              .sendPingsAutomatically = false,

                              .message =
                                [](auto* ws, std::string_view message, uWS::OpCode opCode) {
                                  // parse json from message
                                  nlohmann::json j = nlohmann::json::parse(message);
                                  auto now = std::chrono::duration_cast<std::chrono::milliseconds>(
                                               std::chrono::system_clock::now().time_since_epoch())
                                               .count();
                                  auto latency = now - j["created_at"].get<long>();
                                  for (int index = 0; index < WRITE_FACTOR; index++) {
                                    nlohmann::json server_msg = {
                                      {"client_id", j["client_id"]},
                                      {"msg_id", j["msg_id"]},
                                      {"msg", j["msg"]},
                                      {"created_at", now},
                                      {"client_ts", j["created_at"]},
                                      {"server_latency", latency},
                                    };
                                    ws->send(server_msg.dump());
                                  }
                                }})
    .listen(3000,
            [](auto* listen_socket) {
              if (listen_socket) {
                std::cout << "Listening on port " << 3000 << std::endl;
              }
            })
    .run();
}