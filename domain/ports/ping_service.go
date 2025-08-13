package ports

import (
	"context"
	"mtu-optimizer/domain/entities"
	"time"
)

type PingService interface {
	Ping(ctx context.Context, host string, mtu int, timeout time.Duration) (*entities.PingResult, error)
	PingMultiple(ctx context.Context, host string, mtu int, count int, timeout time.Duration) ([]entities.PingResult, error)
}

type MTUOptimizer interface {
	TestMTU(ctx context.Context, host string, mtu int, pingCount int, timeout time.Duration) (*entities.MTUTestResult, error)
	OptimizeRange(ctx context.Context, host string, mtuRange entities.MTURange, pingCount int, timeout time.Duration) (*entities.OptimizationResult, error)
	OptimizeRangeWithProgress(ctx context.Context, host string, mtuRange entities.MTURange, pingCount int, timeout time.Duration, progressCallback entities.ProgressCallback) (*entities.OptimizationResult, error)
}
