defmodule Router.MixProject do
  use Mix.Project

  def project do
    [
      app: :router,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {Router.Application, []}
    ]
  end

  defp deps do
    [
      {:grpc, "~> 0.7.0"},           # gRPC server (elixir-grpc)
      {:protobuf, "~> 0.10.0"},      # Proto message support
      {:kafka_ex, "~> 0.13.0"},      # Kafka/Redpanda client
      {:jason, "~> 1.4"}             # JSON serialization
    ]
  end
end
