#!/usr/bin/python3

import sys
import os
import re

def main(argv):

    TARGET_FILE = "db_init.sql"
    WORKING_PATH = "migrations"
    REGEX = "up.sql"
    
    if len(argv) > 0:
        WORKING_PATH = str(argv[0])

    regex = re.compile(REGEX)
    target = open(os.path.join(WORKING_PATH, TARGET_FILE), "w")
    
    for root, dirs, files in os.walk(WORKING_PATH):
        for file in files:
            if regex.match(file):
                path = os.path.join(root, file)
                print("Reading content from", path)

                fo = open(path, "r")
                content = fo.read()
                fo.close()

                target.write(content)
                target.write("\n")
    
    target.close()

if __name__ == "__main__":
   main(sys.argv[1:])