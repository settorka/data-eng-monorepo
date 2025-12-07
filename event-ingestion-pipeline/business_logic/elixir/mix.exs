defmodule BusinessLogic.MixProject do
  use Mix.Project

  def project do
    [
      app: :business_logic,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger],
      mod: {BusinessLogic.Application, []}
    ]
  end

  defp deps do
    [
      {:grpc, "~> 0.7"},
      {:protobuf, "~> 0.12"},
      {:jason, "~> 1.4"},
      {:uuid, "~> 1.1"} 
    ]
  end
end
