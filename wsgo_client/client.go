package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/url"
	"os"
	"os/signal"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

const WRITE_FACTOR = 4

var addr = flag.String("s", "localhost:3000", "http service address")
var nclients = flag.Int("nc", 10, "number of simultanous clients")
var batchSize = flag.Int("bs", 10, "number of msgs to send in on batch")
var nbatch = flag.Int("nb", 100, "number of batchs")
var wait = flag.Int("w", 100, "wait period in (ms) between batches")

type ClientMessage struct {
	ClientID  int    `json:"client_id"`
	MsgID     int    `json:"msg_id"`
	Msg       string `json:"msg"`
	CreatedAt uint64 `json:"created_at"`
}

type ServerMessage struct {
	ClientID      int    `json:"client_id"`
	MsgID         int    `json:"msg_id"`
	Msg           string `json:"msg"`
	CreatedAt     int    `json:"created_at"`
	ClientTS      int    `json:"client_ts"`
	ServerLatency int    `json:"server_latency"`
}

func runClient(id int, wg *sync.WaitGroup, url string, nBatch int, batchSize int, wait int, interrupt chan bool) {
	defer wg.Done()

	dialer := websocket.Dialer{
		HandshakeTimeout: 60 * time.Second,
		ReadBufferSize:   1024,
		WriteBufferSize:  1024,
	}
	c, _, err := dialer.Dial(url, nil)
	if err != nil {
		log.Fatal("dial:", err)
	}
	defer c.Close()

	done := make(chan struct{})

	//  Receiver part
	go func() {
		var serverMsg ServerMessage
		rcvCnt := 0
		for rcvCnt < batchSize*nBatch*WRITE_FACTOR {
			_, message, err := c.ReadMessage()
			if err != nil {
				log.Printf("Closing receiver. received: %d msg\n", rcvCnt)
				return
			}

			if json.Unmarshal(message, &serverMsg) != nil {
				fmt.Println("Error:", err)
				return
			}
			rcvCnt += 1
		}
		err := c.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
		if err != nil {
			log.Println("write close:", err)
			return
		}

		close(done)
	}()

	// Sender part
	ticker := time.NewTicker(time.Duration(wait) * time.Millisecond)
	cnt := 0

	for i := 0; i < nBatch; i++ {
		select {
		case <-done:
			return
		case <-ticker.C:
			clientMsgBytes, _ := json.Marshal(
				ClientMessage{
					ClientID:  id,
					MsgID:     cnt,
					Msg:       "This is a test message",
					CreatedAt: uint64(time.Now().UnixMilli()),
				})
			for i := 0; i < batchSize; i++ {
				err := c.WriteMessage(websocket.BinaryMessage, clientMsgBytes)
				if err != nil {
					log.Println("write:", err)
					return
				}
				// Increment msg Id
				cnt += 1
			}
		case <-interrupt:
			err := c.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
			if err != nil {
				log.Println("write close:", err)
				return
			}
			select {
			case <-done:
			case <-time.After(time.Second):
			}
			return
		}

		ticker.Stop()
	}

	<-done
}

func main() {
	flag.Parse()
	log.SetFlags(0)

	var wg sync.WaitGroup

	u := url.URL{Scheme: "ws", Host: *addr, Path: "/ws"}
	log.Printf("connecting to %s", u.String())

	// Setup interrupts
	interrupt_main := make(chan os.Signal, 1)
	signal.Notify(interrupt_main, os.Interrupt)

	interrupts := make(chan bool, *nclients)
	go func() {

		<-interrupt_main
		log.Println("Received interrupt")
		for i := 0; i < *nclients; i++ {
			interrupts <- true

		}
	}()

	for i := 1; i <= *nclients; i++ {
		wg.Add(1)
		go runClient(i, &wg, u.String(), *batchSize, *nbatch, *wait, interrupts)
	}

	wg.Wait()

}
