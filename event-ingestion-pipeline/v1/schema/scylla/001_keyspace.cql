CREATE KEYSPACE IF NOT EXISTS events
WITH replication = {
  'class': 'SimpleStrategy',
  'replication_factor': 1
};