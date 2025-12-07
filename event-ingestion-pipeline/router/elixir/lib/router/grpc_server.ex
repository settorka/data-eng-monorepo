defmodule Router.GRPCServer do
  @moduledoc """
  gRPC Router service implementation.
  Receives events from the ingress and forwards them to the handler.
  """

  use GRPC.Server, service: Chat.Router.Service
  require Logger

  alias Router.EventHandler
  alias Chat.{Event, IngestResponse}

  @spec ingest(Event.t(), GRPC.Server.Stream.t()) :: IngestResponse.t()
  def ingest(event, _stream) do
    Logger.info("Received event: #{inspect(event.kind)}")

    case EventHandler.handle_event(event) do
      :ok ->
        Logger.info("Event processed successfully")
        IngestResponse.new(ok: true, error: "")

      {:error, reason} ->
        Logger.error("Failed to process event: #{inspect(reason)}")
        IngestResponse.new(ok: false, error: reason)
    end
  end
end
