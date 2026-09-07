#!/usr/bin/env bash
set -euo pipefail

# Find unpaid invoices due before 2026-09-01 in a billing CSV export.
# A fixed review date makes the example reproducible. An invoice due on that
# date is not overdue yet. CSV parsing preserves commas inside customer names.
# Columns: invoice, customer, amount_jpy, due_date, status, owner
cho --csv --skip-header '
  (filter (s/= $5 "unpaid"))
  (filter (d/< $4 "2026-09-01"))
  (print $1 "|" $2 "|" (str $3 " JPY") "|"
    (str "due=" $4) "|" (str "owner=" (default $6 "unassigned")))
' <<'CSV'
invoice,customer,amount_jpy,due_date,status,owner
INV-101,Maple Studio,48000,2026-08-20,unpaid,Aoki
INV-102,Harbor Books,12000,2026-08-15,paid,Sato
INV-103,"Northwind, Inc.",96000,2026-08-31,unpaid,
INV-104,Sunny Cafe,18000,2026-09-01,unpaid,Sato
CSV

# Expected output:
# INV-101 | Maple Studio | 48000 JPY | due=2026-08-20 | owner=Aoki
# INV-103 | Northwind, Inc. | 96000 JPY | due=2026-08-31 | owner=unassigned
