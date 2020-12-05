CREATE USER 'backend'@'%' IDENTIFIED BY 'backendpwd';
GRANT ALL ON SafeEvents.* TO 'backend'@'%';