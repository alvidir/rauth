#!/usr/bin/python3

import os
import re

TARGET_DIR = ".postgres"
TARGET_FILE = "setup.sql"
WORKING_PATH = "migrations"
REGEX = "up.sql"

dir = os.path.join(WORKING_PATH, TARGET_DIR)
if not os.path.exists(dir):
    os.mkdir(dir)

regex = re.compile(REGEX)
target = open(os.path.join(dir, TARGET_FILE), "w")

for root, dirs, files in os.walk(WORKING_PATH):
    for file in files:
        if regex.match(file):
            path = os.path.join(root, file)
            print("Reading content from {}".format(path))

            fo = open(path, "r")
            content = fo.read()
            fo.close()

            target.write(content)
            target.write("\n")

target.close()