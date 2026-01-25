//+------------------------------------------------------------------+
//|                                                 ZmqPublisher.mq5 |
//|                                    Copyright 2024, Antigravity   |
//|                                                                  |
//+------------------------------------------------------------------+
#property copyright "Antigravity"
#property link      ""
#property version   "1.00"

// Include our ZMQ wrapper
#include <Zmq/Zmq.mqh>

// Input parameters
input string InpBindAddress = "tcp://0.0.0.0:5555"; // Bind Address

CZmq *g_publisher;

//+------------------------------------------------------------------+
//| Expert initialization function                                   |
//+------------------------------------------------------------------+
int OnInit()
  {
   Print("Initializing ZmqPublisher...");
   
   g_publisher = new CZmq();
   
   if(!g_publisher.Init(ZMQ_PUB)) {
      Print("Failed to initialize ZMQ Publisher");
      return(INIT_FAILED);
   }
   
   if(!g_publisher.Bind(InpBindAddress)) {
      Print("Failed to bind to ", InpBindAddress);
      return(INIT_FAILED);
   }
   
   Print("ZmqPublisher bound to ", InpBindAddress);
   return(INIT_SUCCEEDED);
  }

//+------------------------------------------------------------------+
//| Expert deinitialization function                                 |
//+------------------------------------------------------------------+
void OnDeinit(const int reason)
  {
   Print("Deinitializing ZmqPublisher...");
   if(g_publisher != NULL) {
      g_publisher.Shutdown();
      delete g_publisher;
      g_publisher = NULL;
   }
  }

//+------------------------------------------------------------------+
//| Expert tick function                                             |
//+------------------------------------------------------------------+
void OnTick()
  {
   if(g_publisher == NULL) return;
   
   MqlTick tick;
   if(SymbolInfoTick(_Symbol, tick)) {
      // Create JSON string manually to avoid complex dependencies
      // Format: {"symbol": "XAUUSD", "bid": 2000.5, "ask": 2000.8, "time": 123456789}
      
      string json;
      StringConcatenate(json, "{\"symbol\":\"", _Symbol, 
                        "\",\"bid\":", DoubleToString(tick.bid, _Digits),
                        ",\"ask\":", DoubleToString(tick.ask, _Digits),
                        ",\"time\":", IntegerToString(tick.time),
                        "}");
                        
      g_publisher.Send(json);
      // Print("Published: ", json); // Uncomment for debugging (spammy)
   }
  }
//+------------------------------------------------------------------+
