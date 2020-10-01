package user

// NewUser builds a brand new user as client
func NewUser(pwd string) User {
	return &user{}
}
