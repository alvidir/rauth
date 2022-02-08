#!/usr/bin/python3

import os
import re
import sys

from dotenv import load_dotenv
load_dotenv() 

WORKDIR = os.getenv("DB_MIGRATIONS_PATH")
TARGET = os.getenv("DB_SETUP_SCRIPT_PATH")
REGEX = os.getenv("DB_FILE_REGEX")

regex = re.compile(REGEX)

def is_migration_files(filename) -> bool:
    return regex.match(filename)

def main() -> int:
    print("Browsing for migration files...")
    
    target_dirname = os.path.dirname(TARGET)
    if not os.path.exists(target_dirname):
        os.mkdir(target_dirname)

    scripts = []
    
    for root, _, files in os.walk(WORKDIR):
        files = filter(is_migration_files, files)
        
        def make_absolute_path(filename) -> str:
            return os.path.join(root, filename)

        files = map(make_absolute_path, files)
        scripts += list(files)

    if not scripts:
        print("No migration files where found")
        return 1

    target = open(TARGET, "w")
    for path in sorted(scripts):
        print("-\t{}".format(path))

        fo = open(path, "r")
        content = fo.read()
        fo.close()

        target.write("{}\n".format(content))

    target.close()
    return 0

if __name__ == '__main__':
    sys.exit(main())