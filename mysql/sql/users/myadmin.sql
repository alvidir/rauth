CREATE USER '<myadmin-user>'@'%' IDENTIFIED BY '<myadmin-password>';
GRANT ALL PRIVILEGES ON *.* TO '<myadmin-user>'@'%' WITH GRANT OPTION;