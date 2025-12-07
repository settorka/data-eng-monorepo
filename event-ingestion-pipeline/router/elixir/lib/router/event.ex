defmodule Router.Event do
  @moduledoc """
  Utility functions for event enrichment and normalization.
  Adds UUIDs, server timestamps, and type markers.
  """

  def enrich(event, event_type) do
    base = Map.from_struct(event)
    Map.merge(base, %{
      event_type: event_type,
      event_id: UUID.uuid4(),
      server_timestamp: System.system_time(:millisecond)
    })
  end
end
