package user

// NewUser builds a brand new user as client
func NewUser(nickname, email string) *User {
	return &User{
		Nickname: nickname,
		Emails:   []string{email},
	}
}
