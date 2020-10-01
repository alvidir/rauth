package user

type user struct {
	name string
}

func (user *user) GetName() string {
	return user.name
}
