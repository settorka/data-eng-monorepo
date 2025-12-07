defmodule BusinessLogic.EventEnricher do
  @moduledoc """
  Adds IDs, server timestamps, and other metadata
  to event fields before they are converted to ProcessedEvent.
  """

  def enrich(fields) when is_map(fields) do
    Map.merge(fields, %{
      event_id: UUID.uuid4(),
      server_timestamp: System.system_time(:millisecond)
    })
  end
end
