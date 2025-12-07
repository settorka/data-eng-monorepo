defmodule Router.EventHandler do
  @moduledoc """
  Handles business logic for all incoming events.
  Validates, enriches, and relays events to Redpanda via Router.Producer.
  """

  require Logger
  alias Router.{Producer, Event}
  alias Chat.Event

  def handle_event(%Event{kind: {:chat, chat_event}}), do: handle_chat(chat_event)
  def handle_event(%Event{kind: {:join, join_event}}), do: handle_join(join_event)
  def handle_event(%Event{kind: {:leave, leave_event}}), do: handle_leave(leave_event)
  def handle_event(%Event{kind: {:reaction, reaction_event}}), do: handle_reaction(reaction_event)
  def handle_event(_), do: {:error, "unknown or malformed event"}

  # Chat message
  defp handle_chat(chat) do
    with :ok <- validate_chat(chat),
         enriched <- Event.enrich(chat, "chat"),
         :ok <- Producer.produce("chat-events", enriched) do
      :ok
    else
      {:error, reason} ->
        {:error, reason}
    end
  end

  # Join event
  defp handle_join(join) do
    with :ok <- validate_join(join),
         enriched <- Event.enrich(join, "join"),
         :ok <- Producer.produce("chat-events", enriched) do
      :ok
    else
      {:error, reason} ->
        {:error, reason}
    end
  end

  # Leave event
  defp handle_leave(leave) do
    with :ok <- validate_leave(leave),
         enriched <- Event.enrich(leave, "leave"),
         :ok <- Producer.produce("chat-events", enriched) do
      :ok
    else
      {:error, reason} ->
        {:error, reason}
    end
  end

  # Reaction event
  defp handle_reaction(reaction) do
    with :ok <- validate_reaction(reaction),
         enriched <- Event.enrich(reaction, "reaction"),
         :ok <- Producer.produce("chat-events", enriched) do
      :ok
    else
      {:error, reason} ->
        {:error, reason}
    end
  end

  ## VALIDATION HELPERS

  defp validate_chat(%{user_id: nil}), do: {:error, "user_id missing"}
  defp validate_chat(%{room_id: nil}), do: {:error, "room_id missing"}
  defp validate_chat(%{message: nil}), do: {:error, "message missing"}
  defp validate_chat(_), do: :ok

  defp validate_join(%{user_id: nil}), do: {:error, "user_id missing"}
  defp validate_join(%{room_id: nil}), do: {:error, "room_id missing"}
  defp validate_join(_), do: :ok

  defp validate_leave(%{user_id: nil}), do: {:error, "user_id missing"}
  defp validate_leave(%{room_id: nil}), do: {:error, "room_id missing"}
  defp validate_leave(_), do: :ok

  defp validate_reaction(%{user_id: nil}), do: {:error, "user_id missing"}
  defp validate_reaction(%{room_id: nil}), do: {:error, "room_id missing"}
  defp validate_reaction(%{message_id: nil}), do: {:error, "message_id missing"}
  defp validate_reaction(%{emoji: nil}), do: {:error, "emoji missing"}
  defp validate_reaction(_), do: :ok
end
