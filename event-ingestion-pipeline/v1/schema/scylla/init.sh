#!/bin/bash
set -e

until cqlsh scylladb 9042 -e "describe keyspaces"; do
  echo "waiting for scylla..."
  sleep 2
done

cqlsh scylladb 9042 -f /schema/001_keyspace.cql
cqlsh scylladb 9042 -f /schema/002_events_table.cql