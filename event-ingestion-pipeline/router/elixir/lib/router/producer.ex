defmodule Router.Producer do
  @moduledoc """
  Handles Redpanda (Kafka protocol) message production.
  """

  require Logger
  alias KafkaEx.Protocol.Produce
  alias KafkaEx

  @topic "chat-events"

  @spec produce(binary(), map()) :: :ok | {:error, String.t()}
  def produce(topic \\ @topic, event) do
    try do
      payload = Jason.encode!(event)
      key = Map.get(event, :room_id, "unknown")

      req = %Produce.Request{
        topic: topic,
        required_acks: 1,
        partition: 0,
        messages: [%Produce.Message{key: key, value: payload}]
      }

      case KafkaEx.produce(req) do
        :ok ->
          Logger.debug("Produced event to #{topic}: #{key}")
          :ok

        {:error, reason} ->
          Logger.error("Failed to produce to Redpanda: #{inspect(reason)}")
          {:error, "redpanda error"}
      end
    rescue
      e ->
        Logger.error("Producer exception: #{inspect(e)}")
        {:error, "serialization failure"}
    end
  end
end
