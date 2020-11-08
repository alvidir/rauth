package user

// New builds a brand new user as client
func New(nickname, email, pwd string) *User {
	var usr User
	usr.Nickname = nickname
	usr.Emails = []string{email}
	usr.Nickname = nickname
	//usr.Client = client.New(&usr, pwd)
	return &usr
}
