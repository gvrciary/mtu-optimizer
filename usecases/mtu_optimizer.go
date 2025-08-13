package usecases

import (
	"context"
	"fmt"
	"mtu-optimizer/domain/entities"
	"mtu-optimizer/domain/ports"
	"sort"
	"time"
)

type MTUOptimizerUseCase struct {
	pingService ports.PingService
}

func NewMTUOptimizerUseCase(pingService ports.PingService) ports.MTUOptimizer {
	return &MTUOptimizerUseCase{
		pingService: pingService,
	}
}

func (m *MTUOptimizerUseCase) TestMTU(ctx context.Context, host string, mtu int, pingCount int, timeout time.Duration) (*entities.MTUTestResult, error) {
	pings, err := m.pingService.PingMultiple(ctx, host, mtu, pingCount, timeout)
	if err != nil {
		return nil, err
	}

	result := &entities.MTUTestResult{
		MTU:   mtu,
		Pings: pings,
	}

	m.calculateStatistics(result)
	return result, nil
}

func (m *MTUOptimizerUseCase) OptimizeRange(ctx context.Context, host string, mtuRange entities.MTURange, pingCount int, timeout time.Duration) (*entities.OptimizationResult, error) {
	return m.OptimizeRangeWithProgress(ctx, host, mtuRange, pingCount, timeout, nil)
}

func (m *MTUOptimizerUseCase) OptimizeRangeWithProgress(ctx context.Context, host string, mtuRange entities.MTURange, pingCount int, timeout time.Duration, progressCallback entities.ProgressCallback) (*entities.OptimizationResult, error) {
	startTime := time.Now()
	var allResults []entities.MTUTestResult
	totalPackets := 0

	totalMTUs := (mtuRange.Max-mtuRange.Min)/mtuRange.Step + 1
	currentMTUIndex := 0

	for mtu := mtuRange.Min; mtu <= mtuRange.Max; mtu += mtuRange.Step {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
		}

		if progressCallback != nil {
			progressCallback(currentMTUIndex, totalMTUs, fmt.Sprintf("Testing MTU %d", mtu))
		}

		result, err := m.TestMTU(ctx, host, mtu, pingCount, timeout)
		if err != nil {
			currentMTUIndex++
			continue
		}

		allResults = append(allResults, *result)
		totalPackets += len(result.Pings)
		currentMTUIndex++
	}

	if progressCallback != nil {
		progressCallback(totalMTUs, totalMTUs, "Calculating results...")
	}

	if len(allResults) == 0 {
		return &entities.OptimizationResult{
			AllResults:    allResults,
			TestedRange:   mtuRange,
			TotalPackets:  totalPackets,
			TotalDuration: time.Since(startTime),
		}, nil
	}

	bestMTU := m.findBestMTU(allResults)
	worstMTU := m.findWorstMTU(allResults)

	return &entities.OptimizationResult{
		BestMTU:       bestMTU,
		WorstMTU:      worstMTU,
		AllResults:    allResults,
		OptimalMTU:    bestMTU.MTU,
		TestedRange:   mtuRange,
		TotalPackets:  totalPackets,
		TotalDuration: time.Since(startTime),
	}, nil
}

func (m *MTUOptimizerUseCase) calculateStatistics(result *entities.MTUTestResult) {
	if len(result.Pings) == 0 {
		return
	}

	var validPings []entities.PingResult
	var totalLatency time.Duration
	lossCount := 0

	for _, ping := range result.Pings {
		if ping.Success {
			validPings = append(validPings, ping)
			totalLatency += ping.Latency
		} else {
			lossCount++
		}
	}

	if len(validPings) == 0 {
		result.PacketLossRate = 1.0
		return
	}

	result.Success = true
	result.AvgLatency = totalLatency / time.Duration(len(validPings))
	result.PacketLossRate = float64(lossCount) / float64(len(result.Pings))

	sort.Slice(validPings, func(i, j int) bool {
		return validPings[i].Latency < validPings[j].Latency
	})

	result.MinLatency = validPings[0].Latency
	result.MaxLatency = validPings[len(validPings)-1].Latency

	if len(validPings) > 1 {
		var jitterSum time.Duration
		for i := 1; i < len(validPings); i++ {
			diff := validPings[i].Latency - validPings[i-1].Latency
			if diff < 0 {
				diff = -diff
			}
			jitterSum += diff
		}
		result.Jitter = jitterSum / time.Duration(len(validPings)-1)
	}
}

func (m *MTUOptimizerUseCase) findBestMTU(results []entities.MTUTestResult) *entities.MTUTestResult {
	var best *entities.MTUTestResult

	for i := range results {
		result := &results[i]
		if !result.Success {
			continue
		}

		if best == nil {
			best = result
			continue
		}

		if result.PacketLossRate < best.PacketLossRate {
			best = result
		} else if result.PacketLossRate == best.PacketLossRate {
			if result.AvgLatency < best.AvgLatency {
				best = result
			} else if result.AvgLatency == best.AvgLatency && result.MTU > best.MTU {
				best = result
			}
		}
	}

	return best
}

func (m *MTUOptimizerUseCase) findWorstMTU(results []entities.MTUTestResult) *entities.MTUTestResult {
	var worst *entities.MTUTestResult

	for i := range results {
		result := &results[i]
		if !result.Success {
			if worst == nil || result.PacketLossRate > worst.PacketLossRate {
				worst = result
			}
			continue
		}

		if worst == nil {
			worst = result
			continue
		}

		if result.PacketLossRate > worst.PacketLossRate {
			worst = result
		} else if result.PacketLossRate == worst.PacketLossRate {
			if result.AvgLatency > worst.AvgLatency {
				worst = result
			}
		}
	}

	return worst
}
