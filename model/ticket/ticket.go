package ticket

import (
	"time"

	"gorm.io/gorm"
)

// Ticket represents an important action over a client data
type Ticket struct {
	gorm.Model
	ID          uint      `json:"id" gorm:"primaryKey; autoIncrement:true"`
	Kind        Kind      `json:"kind" gorm:"not null"`
	Description string    `json:"description"`
	ConfirmURL  string    `json:"confirmation_url" gorm:"not null"`
	CreatedAt   time.Time `json:"createdAt"`
	Deadline    time.Time `json:"deadline"`
}
