package credential

import "time"

type credential struct {
	public   string
	creation time.Time
	deadline time.Time
}

func (cred *credential) GetPublic() string {
	return cred.public
}
