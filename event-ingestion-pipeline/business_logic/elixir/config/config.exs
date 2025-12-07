import Config

config :kafka_ex,
  brokers: [{"localhost", 9092}],
  consumer_group: "router-group",
  use_ssl: false

config :logger,
  level: :info,
  truncate: :infinity
