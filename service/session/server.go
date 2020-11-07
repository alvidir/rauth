package session

import (
	"context"

	pb "github.com/alvidir/tp-auth/proto/client"
	tx "github.com/alvidir/tp-auth/service/session/transactions"
	"google.golang.org/grpc"
)

// session represents a service for session
type session struct {
	pb.UnimplementedSessionServer
}

// RegisterServer registers the current session service into a provided grpc server
func (session *session) RegisterServer(grpcServer *grpc.Server) {
	pb.RegisterSessionServer(grpcServer, session)
}

// Signup implementation for the protobuf contract
func (session *session) Signup(ctx context.Context, req *pb.SignupRequest) (out *pb.SessionResponse, err error) {
	txSignup := tx.NewTxSignup(req)
	txSignup.Execute(ctx)

	out = &pb.SessionResponse{
		Cookie:   "",
		Deadline: 0,
		Status:   pb.Status_ALIVE,
	}

	return
}

// Login implementation for the protobuf contract
func (session *session) Login(ctx context.Context, req *pb.LoginRequest) (out *pb.SessionResponse, err error) {
	txLogin := tx.NewTxLogin(req)
	txLogin.Execute(ctx)

	out = &pb.SessionResponse{
		Cookie:   "",
		Deadline: 0,
		Status:   pb.Status_ALIVE,
	}

	return
}

// GoogleSignin implementation for the protobuf contract
func (session *session) GoogleSignin(ctx context.Context, req *pb.GoogleSigninRequest) (out *pb.SessionResponse, err error) {
	txGoogleSignin := tx.NewTxGoogleSignin(req)
	txGoogleSignin.Execute(ctx)

	out = &pb.SessionResponse{
		Cookie:   "",
		Deadline: 0,
		Status:   pb.Status_ALIVE,
	}

	return
}

// Logout implementation for the protobuf contract
func (session *session) Logout(ctx context.Context, req *pb.LogoutRequest) (out *pb.SessionResponse, err error) {
	txLogout := tx.NewTxLogout(req)
	txLogout.Execute(ctx)

	out = &pb.SessionResponse{
		Cookie:   "",
		Deadline: 0,
		Status:   pb.Status_ALIVE,
	}

	return
}
