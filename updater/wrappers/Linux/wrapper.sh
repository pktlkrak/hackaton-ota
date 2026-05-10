#!/bin/bash

# This file's job is to run the updater before starting the application.
# The updater will either pull the update file and return a predefined error code to
# signal the fact that the updater got pulled, or it will exit normally, which should
# start the app without any updates being triggered.

source ./app_config
SERVER=https://192.168.1.75:8443/


./updater --current-version $VERSION \
          --key-directory keys \
          check \
          --installer-to-write installer.tmp \
          --serial $SERIAL \
          --cert-dir certificates \
          $SERVER

if [ $? -eq 123 ]; then
    echo "Running"
    chmod a+x installer.tmp
    ./installer.tmp
    rm installer.tmp
fi

./application
