package ticket

import "time"

// New returns a brand new ticket
func New(kind Kind, deadline time.Time, description, url string) *Ticket {
	return &Ticket{
		Kind:        kind,
		Description: description,
		CreatedAt:   time.Now(),
		Deadline:    deadline,
	}
}
