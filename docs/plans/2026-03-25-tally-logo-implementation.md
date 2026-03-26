# Tally Logo Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a new standalone SVG logo asset for Tally using a warm, minimal data-mark.

**Architecture:** Create a single SVG file in the shared assets folder. Keep the logo self-contained with inline shapes only so it is easy to reuse in React, docs, or export workflows.

**Tech Stack:** SVG

---

### Task 1: Create the SVG logo asset

**Files:**
- Create: `src/assets/tally-logo.svg`

**Step 1: Build the base tile**

Create a rounded square mark with Tally's cream background and a soft border.

**Step 2: Add the data grid**

Add a subtle 3x3 rounded-cell grid inside the tile.

**Step 3: Add highlighted activity cells**

Fill a few cells with terracotta at increasing emphasis to imply usage patterns over time.

### Task 2: Validate the SVG

**Files:**
- No code changes required unless fixes are needed

**Step 1: Check SVG syntax**

Run a lightweight validation command if available.

**Step 2: Confirm file location**

Make sure the asset lives under `src/assets` for easy reuse.
