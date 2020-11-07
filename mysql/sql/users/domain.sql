CREATE USER '<service_name>'@'%' IDENTIFIED BY '<service_password>';
GRANT ALL ON tpAuth.* TO '<service_name>'@'%';