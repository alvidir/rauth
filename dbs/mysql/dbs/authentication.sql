ALTER USER 'root'@'localhost' IDENTIFIED BY 'rootpwd';

CREATE USER 'myadmin'@'%' IDENTIFIED BY 'myadminpwd';
GRANT ALL PRIVILEGES ON *.* TO 'myadmin'@'%' WITH GRANT OPTION;

CREATE DATABASE 'tp-auth' IF NOT EXISTS;