defmodule BusinessLogic.Application do
  @moduledoc """
  Entry point and supervision tree for the BusinessLogic service.
  Boots the gRPC server and manages its lifecycle.
  """

  use Application
  require Logger

  @impl true
  def start(_type, _args) do
    port = 50_051
    Logger.info("Starting BusinessLogic gRPC server on port #{port}")

    children = [
      {GRPC.Server.Supervisor, {BusinessLogic.GRPCServer, port}}
    ]

    opts = [strategy: :one_for_one, name: BusinessLogic.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
