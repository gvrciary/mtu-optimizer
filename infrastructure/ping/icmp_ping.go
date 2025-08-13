package ping

import (
	"context"
	"mtu-optimizer/domain/entities"
	"mtu-optimizer/domain/ports"
	"net"
	"os/exec"
	"regexp"
	"strconv"
	"strings"
	"time"
)

type ICMPPingService struct{}

func NewICMPPingService() ports.PingService {
	return &ICMPPingService{}
}

func (s *ICMPPingService) Ping(ctx context.Context, host string, mtu int, timeout time.Duration) (*entities.PingResult, error) {
	payloadSize := mtu - 28

	if payloadSize <= 0 {
		return &entities.PingResult{
			MTU:         mtu,
			PacketLoss:  true,
			Success:     false,
			PayloadSize: payloadSize,
			Timestamp:   time.Now(),
		}, nil
	}

	start := time.Now()

	cmd := exec.CommandContext(ctx, "ping",
		"-c", "1",
		"-s", strconv.Itoa(payloadSize),
		"-W", strconv.Itoa(int(timeout.Milliseconds())),
		host)

	output, err := cmd.CombinedOutput()
	timestamp := time.Now()

	result := &entities.PingResult{
		MTU:         mtu,
		PayloadSize: payloadSize,
		Timestamp:   timestamp,
	}

	if err != nil {
		result.PacketLoss = true
		result.Success = false
		return result, nil
	}

	latency, success := s.parseLatency(string(output))
	result.Latency = latency
	result.Success = success
	result.PacketLoss = !success

	if !success && strings.Contains(string(output), "Packet too large") {
		result.PacketLoss = true
	}

	if result.Success && result.Latency == 0 {
		result.Latency = time.Since(start)
	}

	return result, nil
}

func (s *ICMPPingService) PingMultiple(ctx context.Context, host string, mtu int, count int, timeout time.Duration) ([]entities.PingResult, error) {
	results := make([]entities.PingResult, 0, count)

	for i := 0; i < count; i++ {
		select {
		case <-ctx.Done():
			return results, ctx.Err()
		default:
		}

		result, err := s.Ping(ctx, host, mtu, timeout)
		if err != nil {
			return results, err
		}

		results = append(results, *result)

		if i < count-1 {
			time.Sleep(100 * time.Millisecond)
		}
	}

	return results, nil
}

func (s *ICMPPingService) parseLatency(output string) (time.Duration, bool) {
	timeRegex := regexp.MustCompile(`time=([0-9.]+)\s*ms`)
	matches := timeRegex.FindStringSubmatch(output)

	if len(matches) < 2 {
		return 0, false
	}

	ms, err := strconv.ParseFloat(matches[1], 64)
	if err != nil {
		return 0, false
	}

	return time.Duration(ms * float64(time.Millisecond)), true
}

func (s *ICMPPingService) validateHost(host string) error {
	if net.ParseIP(host) != nil {
		return nil
	}

	_, err := net.LookupHost(host)
	return err
}
