defmodule Router.BusinessLogic do
  @moduledoc """
  Converts any incoming Event (chat, join, leave, reaction)
  into a unified ProcessedEvent ready for Rust to publish to Redpanda.
  """

  alias Router.EventEnricher

  alias Chat.{
    Event,
    ChatEvent,
    JoinEvent,
    LeaveEvent,
    ReactionEvent,
    ProcessedEvent
  }

  def process(%Event{kind: {:chat, %ChatEvent{} = evt}}) do
    base_fields = %{
      event_type: "chat",
      user_id: evt.user_id,
      room_id: evt.room_id,
      journey_id: evt.journey_id || "",
      timestamp: evt.timestamp,
      content: evt.message || "",
      metadata_json: Jason.encode!(%{
        message_type: evt.message_type,
        chat_type: evt.chat_type
      })
    }

    build_processed(base_fields)
  end

  def process(%Event{kind: {:join, %JoinEvent{} = evt}}) do
    base_fields = %{
      event_type: "join",
      user_id: evt.user_id,
      room_id: evt.room_id,
      journey_id: "",
      timestamp: evt.timestamp,
      content: "",
      metadata_json: Jason.encode!(%{})
    }

    build_processed(base_fields)
  end

  def process(%Event{kind: {:leave, %LeaveEvent{} = evt}}) do
    base_fields = %{
      event_type: "leave",
      user_id: evt.user_id,
      room_id: evt.room_id,
      journey_id: "",
      timestamp: evt.timestamp,
      content: "",
      metadata_json: Jason.encode!(%{})
    }

    build_processed(base_fields)
  end

  def process(%Event{kind: {:reaction, %ReactionEvent{} = evt}}) do
    base_fields = %{
      event_type: "reaction",
      user_id: evt.user_id,
      room_id: evt.room_id,
      journey_id: "",
      timestamp: evt.timestamp,
      content: evt.emoji,
      metadata_json: Jason.encode!(%{
        message_id: evt.message_id
      })
    }

    build_processed(base_fields)
  end

  defp build_processed(fields) do
    enriched = EventEnricher.enrich(fields)

    %ProcessedEvent{
      event_id: enriched.event_id,
      server_timestamp: enriched.server_timestamp,
      event_type: enriched.event_type,
      user_id: enriched.user_id,
      room_id: enriched.room_id,
      journey_id: enriched.journey_id,
      timestamp: enriched.timestamp,
      content: enriched.content,
      metadata_json: enriched.metadata_json
    }
  end
end
