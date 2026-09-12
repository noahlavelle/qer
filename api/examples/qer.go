// Usage (after `tools/dev/start.sh` or `tools/dev/portforward.sh`):
//
//	go run ./examples -role=producer
//	go run ./examples -role=consumer
package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"time"

	"github.com/noahlavelle/qer/client"
	"github.com/noahlavelle/qer/gen/openapi"
)

func main() {
	baseURL := flag.String("addr", "http://localhost:8080/api/v1", "QER API base URL")
	role := flag.String("role", "producer", "producer or consumer")
	queue := flag.String("queue", "example-queue", "queue name")
	interval := flag.Duration("interval", 10*time.Second, "delay between calls")
	flag.Parse()

	ctx := context.Background()

	switch *role {
	case "producer":
		c, err := client.AttachProducer(ctx, *baseURL)
		if err != nil {
			log.Fatalf("attach producer: %v", err)
		}
		runProducer(ctx, c, *queue, *interval)
	case "consumer":
		c, err := client.AttachConsumer(ctx, *baseURL)
		if err != nil {
			log.Fatalf("attach consumer: %v", err)
		}
		runConsumer(ctx, c, *queue, *interval)
	default:
		log.Fatalf("unknown role %q (want producer or consumer)", *role)
	}
}

func runProducer(ctx context.Context, c *openapi.ClientWithResponses, queue string, interval time.Duration) {
	createRes, err := c.CreateQueueWithResponse(ctx, openapi.CreateQueueJSONRequestBody{Name: queue})
	if err != nil {
		log.Fatalf("create queue: %v", err)
	}
	if createRes.StatusCode() >= 300 && createRes.StatusCode() != 409 {
		log.Fatalf("create queue: unexpected status %s: %s", createRes.Status(), createRes.Body)
	}

	for seq := 0; ; seq++ {
		payload := map[string]any{"seq": seq, "sent_at": time.Now().Format(time.RFC3339)}
		res, err := c.PutJobWithResponse(ctx, queue, openapi.PutJobJSONRequestBody{Payload: payload})
		if err != nil {
			log.Fatalf("put job: %v", err)
		}
		if res.JSON202 == nil {
			log.Fatalf("put job: unexpected status %s: %s", res.Status(), res.Body)
		}
		fmt.Printf("[%s] put job %s (seq=%d)\n", time.Now().Format(time.RFC3339), res.JSON202.JobId, seq)

		time.Sleep(interval)
	}
}

func runConsumer(ctx context.Context, c *openapi.ClientWithResponses, queue string, interval time.Duration) {
	for {
		res, err := c.ReserveJobWithResponse(ctx, queue)
		if err != nil {
			log.Fatalf("reserve job: %v", err)
		}

		switch {
		case res.JSON200 != nil:
			job := res.JSON200
			fmt.Printf("[%s] reserved job %s lease %s: %v\n",
				time.Now().Format(time.RFC3339), job.JobId, job.LeaseId, job.Payload)

			ackRes, err := c.AckJobWithResponse(ctx, queue, openapi.AckJobJSONRequestBody{LeaseId: job.LeaseId})
			if err != nil {
				log.Fatalf("ack job: %v", err)
			}
			if ackRes.StatusCode() >= 300 {
				log.Fatalf("ack job: unexpected status %s: %s", ackRes.Status(), ackRes.Body)
			}
		case res.StatusCode() == 204:
			// Queue empty, nothing to reserve yet.
		default:
			log.Fatalf("reserve job: unexpected status %s: %s", res.Status(), res.Body)
		}

		time.Sleep(interval)
	}
}
