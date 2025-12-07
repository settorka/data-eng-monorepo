defmodule Chat.ChatEvent do
  @moduledoc false

  use Protobuf, protoc_gen_elixir_version: "0.15.0", syntax: :proto3

  field :user_id, 1, type: :string, json_name: "userId"
  field :room_id, 2, type: :string, json_name: "roomId"
  field :journey_id, 3, type: :string, json_name: "journeyId"
  field :timestamp, 4, type: :int64
  field :message, 5, type: :string
  field :message_type, 6, type: :string, json_name: "messageType"
  field :chat_type, 7, type: :string, json_name: "chatType"
end

defmodule Chat.JoinEvent do
  @moduledoc false

  use Protobuf, protoc_gen_elixir_version: "0.15.0", syntax: :proto3

  field :user_id, 1, type: :string, json_name: "userId"
  field :room_id, 2, type: :string, json_name: "roomId"
  field :timestamp, 3, type: :int64
end

defmodule Chat.LeaveEvent do
  @moduledoc false

  use Protobuf, protoc_gen_elixir_version: "0.15.0", syntax: :proto3

  field :user_id, 1, type: :string, json_name: "userId"
  field :room_id, 2, type: :string, json_name: "roomId"
  field :timestamp, 3, type: :int64
end

defmodule Chat.ReactionEvent do
  @moduledoc false

  use Protobuf, protoc_gen_elixir_version: "0.15.0", syntax: :proto3

  field :user_id, 1, type: :string, json_name: "userId"
  field :room_id, 2, type: :string, json_name: "roomId"
  field :message_id, 3, type: :string, json_name: "messageId"
  field :emoji, 4, type: :string
  field :timestamp, 5, type: :int64
end

defmodule Chat.Event do
  @moduledoc false

  use Protobuf, protoc_gen_elixir_version: "0.15.0", syntax: :proto3

  oneof :kind, 0

  field :chat, 1, type: Chat.ChatEvent, oneof: 0
  field :join, 2, type: Chat.JoinEvent, oneof: 0
  field :leave, 3, type: Chat.LeaveEvent, oneof: 0
  field :reaction, 4, type: Chat.ReactionEvent, oneof: 0
end

defmodule Chat.ProcessedEvent do
  @moduledoc false

  use Protobuf, protoc_gen_elixir_version: "0.15.0", syntax: :proto3

  field :event_type, 1, type: :string, json_name: "eventType"
  field :user_id, 2, type: :string, json_name: "userId"
  field :room_id, 3, type: :string, json_name: "roomId"
  field :journey_id, 4, type: :string, json_name: "journeyId"
  field :timestamp, 5, type: :int64
  field :content, 6, type: :string
  field :metadata_json, 7, type: :string, json_name: "metadataJson"
end
