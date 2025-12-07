defmodule Router.GRPCServer do
  @moduledoc """
  Receives raw Event from Rust, applies business logic,
  returns a ProcessedEvent for Rust to publish to Redpanda.
  """

  use GRPC.Server, service: Chat.Router.Service
  require Logger

  alias Router.BusinessLogic
  alias Chat.{Event, ProcessedEvent}

  @spec ingest(Event.t(), GRPC.Server.Stream.t()) :: ProcessedEvent.t()
  def ingest(event, _stream) do
    Logger.info("Received event: #{inspect(event)}")

    processed = BusinessLogic.process(event)

    Logger.info("Returning processed event: #{inspect(processed)}")

    processed
  end
end
