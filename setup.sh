#!/bin/bash

rm data/app.db
touch data/app.db
sqlx migrate run
