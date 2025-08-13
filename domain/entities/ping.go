package entities

import "time"

type PingResult struct {
	MTU         int
	Latency     time.Duration
	PacketLoss  bool
	Success     bool
	PayloadSize int
	Timestamp   time.Time
}

type MTUTestResult struct {
	MTU            int
	Pings          []PingResult
	AvgLatency     time.Duration
	MinLatency     time.Duration
	MaxLatency     time.Duration
	PacketLossRate float64
	Jitter         time.Duration
	Success        bool
}

type OptimizationResult struct {
	BestMTU       *MTUTestResult
	WorstMTU      *MTUTestResult
	AllResults    []MTUTestResult
	OptimalMTU    int
	TestedRange   MTURange
	TotalPackets  int
	TotalDuration time.Duration
}

type MTURange struct {
	Min  int
	Max  int
	Step int
}
