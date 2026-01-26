# Title: 
SUM3API: Using Rust, ZeroMQ, and MetaQuotes Language (MQL5) API Combination to Extract, Communicate, and Externally Project Financial Data from MetaTrader 5 (MT5)

## Author
Albeos Rembrant

## Role/Organization
Independent Researcher

## Date
2026-01-26

# Abstract

MetaTrader 5 (MT5), when connected to a preferred exchange or broker, supports automated algorithmic trading via Expert Advisors (EAs) written in MetaQuotes Language (MQL5). While MetaQuotes LLC provides an official Python integration package, there are limited publicly documented methods to internally extract and externally project MT5 data. To address this gap, this study implements a novel approach combining MQL5 with multiple programming languages through publisher–subscriber ZeroMQ bindings (e.g., MQL5 + ZMQ + Rust, Go, C++, Python, Java, JavaScript, C#, NodeJS). This benchmark-based methodology enables quantitative researchers, feature engineers, and algorithmic traders to develop trading systems leveraging MT5 datafeeds using their preferred programming languages, bypassing the limitations of pure MQL5 EAs.