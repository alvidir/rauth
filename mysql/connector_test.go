package mysql

// import (
// 	"testing"
// )

// func testInit() {
// }

// func TestConnector(t *testing.T) {
// 	testInit()

// 	if err := godotenv.Load("../.env"); err != nil {
// 		t.Fatalf("Got error %s; while loading dotenv", err.Error())
// 	}

// 	stream, err := OpenStream()
// 	if err != nil {
// 		t.Errorf("Got %v error while OpenStream", err.Error())
// 		t.FailNow()
// 	}

// 	//defer stream.ConnPool.Close()

// 	ctx, cancel := context.WithTimeout(context.TODO(), 10*time.Second)
// 	defer cancel()

// 	query := "SELECT * FROM SafeEvents.Users"
// 	var rows *sql.Rows
// 	if rows, err = stream.ConnPool.QueryContext(ctx, query); err != nil {
// 		t.Errorf("Got %v error while executing query", err.Error())
// 		t.FailNow()
// 	}

// 	defer rows.Close()

// 	var id, nickname, email string
// 	if ok := rows.Next(); !ok {
// 		t.Errorf("Got no row while getting values from first row")
// 		t.FailNow()
// 	}

// 	if err := rows.Scan(&id, &nickname, &email); err != nil {
// 		t.Errorf("Got %v error while scanning row's values", err.Error())
// 		t.FailNow()
// 	}
// }
