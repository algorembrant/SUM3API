# Title: 
SUM3API: Using Rust, ZeroMQ, and MetaQuotes Language (MQL5) API Combination to Extract, Communicate, and Externally Project Financial Data from MetaTrader 5 (MT5)

## Author
Albeos Rembrant

## Role/Organization
Independent Researcher

## Date
2026-01-26

# Abstract

The MetaTrader 5 (MT5), connected to your preferred exchange or broker, supports automated algorithmic trading via Expert Advisors (EAs) written in MetaQuotes Language (MQL5). Also, MetaQuotes LLC provides an official Python integration package; there are limited publicly documented methods for internally extracting and externally projecting MT5 financial data, other than that. To address this gap, this study implements a unique approach that bridges MQL5 and Rust via ZeroMQ publisher–subscriber & request-reply bindings. Nothing is compared in this study, and this benchmark-based methodology enables quantitative researchers, feature engineers, and algorithmic traders to develop trading systems leveraging MT5 datafeeds using Rust, bypassing the limitations of pure MQL5 EAs. The methodology is proven through integrating it in one software application (simple trading termminal) demonstrating these low-latency functionality: includes real-time account info (balance, equity, free and used margin), historical data request (OCHL, raw tick), forward data request (live recorded), trade controls (buy and sell either Market, Limit, or Stop orders) with lotsizing and trade cancelation, messenging and notifications (debugging and recent calls), and a live microsecond raw tick-level bid/ask formation chart. All resources are open-source and available on GitHub: https://github.com/algorembrant/SUM3API

# Introduction



## Background

## Problem Statement

## Research Questions

## Objectives

## Significance

# Methodology

## Research Design

## Data Collection

## Data Analysis

## Implementation

# Results

## Findings

## Discussion

# Conclusion

# References

# Appendices
