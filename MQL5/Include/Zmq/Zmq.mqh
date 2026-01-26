//+------------------------------------------------------------------+
//|                                                          Zmq.mqh |
//|                                    Copyright 2026, Algorembrant  |
//|                                                                  |
//+------------------------------------------------------------------+
#property copyright "Algorembrant"
#property link      "https://github.com/algorembrant/SUM3API"
#property version   "2.00"
#property strict

// Define ZMQ constants
#define ZMQ_PUB 1
#define ZMQ_SUB 2
#define ZMQ_REQ 3
#define ZMQ_REP 4

#define ZMQ_NOBLOCK 1

// Import functions from libzmq.dll
// NOTE: Make sure libzmq.dll is in your MQL5/Libraries folder
// Handles are void* (64-bit on x64), so using 'long' works for both 32 (too big but safe) and 64 bit
#import "libzmq.dll"
   long zmq_ctx_new();
   int zmq_ctx_term(long context);
   long zmq_socket(long context, int type);
   int zmq_close(long socket);
   int zmq_bind(long socket, uchar &endpoint[]);
   int zmq_connect(long socket, uchar &endpoint[]);
   int zmq_send(long socket, uchar &buf[], int len, int flags);
   int zmq_recv(long socket, uchar &buf[], int len, int flags);
   int zmq_errno();
#import

class CZmq {
private:
   long m_context;
   long m_socket;
   bool m_initialized;

public:
   CZmq() {
      m_context = 0;
      m_socket = 0;
      m_initialized = false;
   }

   ~CZmq() {
      Shutdown();
   }

   bool Init(int type) {
      if(m_initialized) return true;
      
      m_context = zmq_ctx_new();
      if(m_context == 0) {
         Print("ZMQ Init failed: Context creation error");
         return false;
      }
      
      m_socket = zmq_socket(m_context, type);
      if(m_socket == 0) {
         Print("ZMQ Init failed: Socket creation error");
         return false;
      }
      
      m_initialized = true;
      return true;
   }

   bool Bind(string endpoint) {
      if(!m_initialized) return false;
      
      uchar data[];
      StringToCharArray(endpoint, data, 0, WHOLE_ARRAY, CP_UTF8);
      
      int rc = zmq_bind(m_socket, data);
      if(rc != 0) {
         Print("ZMQ Bind failed. Error: ", zmq_errno());
         return false;
      }
      return true;
   }

   bool Connect(string endpoint) {
      if(!m_initialized) return false;
      
      uchar data[];
      StringToCharArray(endpoint, data, 0, WHOLE_ARRAY, CP_UTF8);
      
      int rc = zmq_connect(m_socket, data);
      if(rc != 0) {
         Print("ZMQ Connect failed. Error: ", zmq_errno());
         return false;
      }
      return true;
   }

   int Send(string message, bool nonBlocking = true) {
      if(!m_initialized) return -1;
      
      uchar data[];
      StringToCharArray(message, data, 0, WHOLE_ARRAY, CP_UTF8);
      // StringToCharArray includes null terminator, we might not want to send it
      // ZMQ messages are just bytes.
      // -1 because array size includes null char, usually we check ArraySize(data)
      int len = ArraySize(data) - 1; 
      if (len < 0) len = 0;

      int flags = 0;
      if(nonBlocking) flags = ZMQ_NOBLOCK;
      
      int bytesSent = zmq_send(m_socket, data, len, flags);
      return bytesSent;
   }

   // Non-blocking receive - returns empty string if no message available
   string Receive(bool nonBlocking = true) {
      if(!m_initialized) return "";
      
      uchar buffer[4096];
      ArrayInitialize(buffer, 0);
      
      int flags = 0;
      if(nonBlocking) flags = ZMQ_NOBLOCK;
      
      int bytesReceived = zmq_recv(m_socket, buffer, ArraySize(buffer) - 1, flags);
      
      if(bytesReceived <= 0) return "";
      
      return CharArrayToString(buffer, 0, bytesReceived, CP_UTF8);
   }

   void Shutdown() {
      if(m_socket != 0) {
         zmq_close(m_socket);
         m_socket = 0;
      }
      if(m_context != 0) {
         zmq_ctx_term(m_context);
         m_context = 0;
      }
      m_initialized = false;
   }
};
