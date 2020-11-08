package ticket

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
	"gorm.io/gorm"
)

// Ticket represents an important action over a client data
type Ticket struct {
	gorm.Model
	ID          uint           `json:"id" gorm:"primaryKey; autoIncrement:true"`
	Kind        Kind           `json:"kind" gorm:"not null"`
	Description string         `json:"description"`
	ConfirmURL  string         `json:"confirmation_url" gorm:"not null"`
	CreatedAt   time.Time      `json:"createdAt"`
	Deadline    time.Time      `json:"deadline"`
	Client      *client.Client `json:"-" gorm:"foreignKey:ClientID"`
	ClientID    uint
}
