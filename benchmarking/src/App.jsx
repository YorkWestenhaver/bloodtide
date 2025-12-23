import React, { useState, useMemo } from 'react'
import {
  LineChart, Line, AreaChart, Area, XAxis, YAxis, CartesianGrid,
  Tooltip, Legend, ResponsiveContainer, ReferenceLine
} from 'recharts'

// Game Balance Formulas
const calculateGameData = (enemyHpExponent) => {
  const data = []

  for (let wave = 1; wave <= 30; wave++) {
    // Calculate cumulative kills to reach this wave (each wave = 50 kills)
    const totalKills = wave * 50

    // Calculate player level based on kills
    // Kills to level: 25 × 1.2^(level-1), solve for level
    let cumulativeKillsNeeded = 0
    let playerLevel = 1
    while (cumulativeKillsNeeded < totalKills && playerLevel < 100) {
      const killsForNextLevel = 25 * Math.pow(1.2, playerLevel - 1)
      cumulativeKillsNeeded += killsForNextLevel
      if (cumulativeKillsNeeded <= totalKills) {
        playerLevel++
      }
    }

    // PLAYER POWER MODEL
    // Creatures acquired: ~0.6 per level (60% of card rolls are creatures)
    const numCreatures = Math.max(1, Math.floor(playerLevel * 0.6))

    // Creature base DPS: 15 damage × 1.0 attack/sec = 15 DPS per creature
    const baseDPS = 15

    // Creature level bonus: +10% per creature level, creatures gain ~1 level per 50 kills they participate in
    // Approximate avg creature level based on kills distributed across creatures
    const avgCreatureLevel = 1 + Math.floor(totalKills / (numCreatures * 50))
    const levelMultiplier = 1 + 0.1 * avgCreatureLevel

    // Crit bonus: average 1.1x multiplier (5% chance of 2x = +10% average)
    const critMult = 1.1

    // Evolution bonus: at ~9 creatures of same type, evolve to 2x base stats (happens around level 15+)
    // Simplified: evolution kicks in around wave 15
    const evolutionMult = playerLevel >= 15 ? 2.0 : 1.0

    // Total Player DPS
    const playerDPS = numCreatures * baseDPS * levelMultiplier * critMult * evolutionMult

    // ENEMY SCALING MODEL
    // Fodder HP: 30 × 1.12^wave (adjustable exponent)
    const enemyHP = 30 * Math.pow(enemyHpExponent, wave)

    // Fodder spawn rate: 0.8 + (wave × 0.1) per second, capped at 4/sec
    const spawnRate = Math.min(4, 0.8 + wave * 0.1)

    // Fodder speed: 80 px/sec
    const enemySpeed = 80

    // Spawn distance: 700px average
    const spawnDistance = 700

    // DERIVED METRICS
    // Time-to-Kill (TTK) = Enemy HP / Player DPS
    const ttk = enemyHP / playerDPS

    // Kill rate = Player DPS / Enemy HP (enemies killed per second)
    const killRate = playerDPS / enemyHP

    // Enemies alive = spawn_rate × (spawn_distance / enemy_speed + TTK)
    const travelTime = spawnDistance / enemySpeed
    const enemiesAlive = spawnRate * (travelTime + ttk)

    // Pressure = spawn_rate / kill_rate (>1 means falling behind)
    const pressure = spawnRate / killRate

    data.push({
      wave,
      playerLevel,
      numCreatures,
      avgCreatureLevel,
      playerDPS: Math.round(playerDPS * 10) / 10,
      enemyHP: Math.round(enemyHP * 10) / 10,
      ttk: Math.round(ttk * 1000) / 1000,
      spawnRate: Math.round(spawnRate * 100) / 100,
      killRate: Math.round(killRate * 1000) / 1000,
      enemiesAlive: Math.round(enemiesAlive),
      pressure: Math.round(pressure * 100) / 100,
      evolutionMult
    })
  }

  return data
}

const CustomTooltip = ({ active, payload, label }) => {
  if (active && payload && payload.length) {
    const data = payload[0].payload
    return (
      <div style={{
        backgroundColor: '#16213e',
        border: '1px solid #0f3460',
        borderRadius: '8px',
        padding: '12px',
        boxShadow: '0 4px 12px rgba(0,0,0,0.5)'
      }}>
        <p style={{ color: '#e94560', fontWeight: 'bold', marginBottom: '8px' }}>
          Wave {label}
        </p>
        <div style={{ fontSize: '12px', lineHeight: '1.6' }}>
          <p><span style={{ color: '#888' }}>Player Level:</span> {data.playerLevel}</p>
          <p><span style={{ color: '#888' }}>Creatures:</span> {data.numCreatures}</p>
          <p><span style={{ color: '#888' }}>Avg Creature Lvl:</span> {data.avgCreatureLevel}</p>
          <p><span style={{ color: '#4ade80' }}>Player DPS:</span> {data.playerDPS.toLocaleString()}</p>
          <p><span style={{ color: '#f87171' }}>Enemy HP:</span> {data.enemyHP.toLocaleString()}</p>
          <p><span style={{ color: '#60a5fa' }}>TTK:</span> {data.ttk.toFixed(3)}s</p>
          <p><span style={{ color: '#fbbf24' }}>Spawn Rate:</span> {data.spawnRate}/s</p>
          <p><span style={{ color: '#34d399' }}>Kill Rate:</span> {data.killRate.toFixed(3)}/s</p>
          <p><span style={{ color: '#a78bfa' }}>Enemies Alive:</span> {data.enemiesAlive}</p>
          <p><span style={{ color: data.pressure > 1 ? '#f87171' : '#4ade80' }}>
            Pressure:</span> {data.pressure.toFixed(2)}
          </p>
          {data.evolutionMult > 1 && (
            <p style={{ color: '#fbbf24' }}>Evolution Active (2x)</p>
          )}
        </div>
      </div>
    )
  }
  return null
}

const ChartContainer = ({ title, children }) => (
  <div style={{
    backgroundColor: '#16213e',
    borderRadius: '12px',
    padding: '20px',
    marginBottom: '20px',
    boxShadow: '0 4px 12px rgba(0,0,0,0.3)'
  }}>
    <h3 style={{
      color: '#e94560',
      marginBottom: '16px',
      fontSize: '16px',
      fontWeight: '600'
    }}>
      {title}
    </h3>
    {children}
  </div>
)

export default function App() {
  const [enemyHpExponent, setEnemyHpExponent] = useState(1.12)

  const data = useMemo(() => calculateGameData(enemyHpExponent), [enemyHpExponent])

  // Calculate balance warnings
  const overwhelmWave = data.find(d => d.pressure > 1)?.wave
  const highEnemyWave = data.find(d => d.enemiesAlive > 500)?.wave
  const fastTtkWave = data.find(d => d.ttk < 0.3)?.wave
  const slowTtkWave = data.find(d => d.ttk > 2)?.wave

  return (
    <div style={{
      backgroundColor: '#1a1a2e',
      minHeight: '100vh',
      padding: '24px'
    }}>
      <div style={{ maxWidth: '1200px', margin: '0 auto' }}>
        {/* Header */}
        <div style={{
          marginBottom: '24px',
          textAlign: 'center'
        }}>
          <h1 style={{
            color: '#e94560',
            fontSize: '28px',
            fontWeight: 'bold',
            marginBottom: '8px'
          }}>
            Bloodtide Balance Visualizer
          </h1>
          <p style={{ color: '#888', fontSize: '14px' }}>
            Game balance curves for Vampire Survivors-style gameplay
          </p>
        </div>

        {/* Controls */}
        <div style={{
          backgroundColor: '#16213e',
          borderRadius: '12px',
          padding: '20px',
          marginBottom: '24px',
          display: 'flex',
          alignItems: 'center',
          gap: '24px',
          flexWrap: 'wrap'
        }}>
          <div style={{ flex: '1 1 300px' }}>
            <label style={{
              color: '#e0e0e0',
              fontSize: '14px',
              display: 'block',
              marginBottom: '8px'
            }}>
              Enemy HP Scaling Exponent: <span style={{ color: '#e94560', fontWeight: 'bold' }}>{enemyHpExponent.toFixed(2)}</span>
            </label>
            <input
              type="range"
              min="1.08"
              max="1.20"
              step="0.01"
              value={enemyHpExponent}
              onChange={(e) => setEnemyHpExponent(parseFloat(e.target.value))}
              style={{
                width: '100%',
                height: '8px',
                borderRadius: '4px',
                cursor: 'pointer',
                accentColor: '#e94560'
              }}
            />
            <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '11px', color: '#666', marginTop: '4px' }}>
              <span>1.08 (Easy)</span>
              <span>1.12 (Default)</span>
              <span>1.20 (Hard)</span>
            </div>
          </div>

          {/* Warnings */}
          <div style={{
            flex: '1 1 400px',
            display: 'flex',
            flexWrap: 'wrap',
            gap: '8px'
          }}>
            {overwhelmWave && (
              <div style={{
                backgroundColor: 'rgba(248, 113, 113, 0.2)',
                border: '1px solid #f87171',
                borderRadius: '6px',
                padding: '6px 12px',
                fontSize: '12px',
                color: '#f87171'
              }}>
                Overwhelmed at Wave {overwhelmWave}
              </div>
            )}
            {highEnemyWave && (
              <div style={{
                backgroundColor: 'rgba(251, 191, 36, 0.2)',
                border: '1px solid #fbbf24',
                borderRadius: '6px',
                padding: '6px 12px',
                fontSize: '12px',
                color: '#fbbf24'
              }}>
                500+ enemies at Wave {highEnemyWave}
              </div>
            )}
            {slowTtkWave && (
              <div style={{
                backgroundColor: 'rgba(248, 113, 113, 0.2)',
                border: '1px solid #f87171',
                borderRadius: '6px',
                padding: '6px 12px',
                fontSize: '12px',
                color: '#f87171'
              }}>
                TTK {">"} 2s at Wave {slowTtkWave}
              </div>
            )}
            {!overwhelmWave && !highEnemyWave && !slowTtkWave && (
              <div style={{
                backgroundColor: 'rgba(74, 222, 128, 0.2)',
                border: '1px solid #4ade80',
                borderRadius: '6px',
                padding: '6px 12px',
                fontSize: '12px',
                color: '#4ade80'
              }}>
                Balance looks healthy through Wave 30
              </div>
            )}
          </div>
        </div>

        {/* Chart 1: Power Curves (Log Scale) */}
        <ChartContainer title="1. Power Curves (Log Scale)">
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={data} margin={{ top: 10, right: 30, left: 10, bottom: 10 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#0f3460" />
              <XAxis
                dataKey="wave"
                stroke="#888"
                tick={{ fill: '#888', fontSize: 11 }}
                label={{ value: 'Wave', position: 'insideBottomRight', offset: -5, fill: '#888', fontSize: 11 }}
              />
              <YAxis
                scale="log"
                domain={['auto', 'auto']}
                stroke="#888"
                tick={{ fill: '#888', fontSize: 11 }}
                tickFormatter={(value) => value >= 1000 ? `${(value/1000).toFixed(0)}k` : value.toFixed(0)}
              />
              <Tooltip content={<CustomTooltip />} />
              <Legend wrapperStyle={{ fontSize: '12px' }} />
              <Line
                type="monotone"
                dataKey="playerDPS"
                stroke="#4ade80"
                strokeWidth={2}
                dot={false}
                name="Player DPS"
              />
              <Line
                type="monotone"
                dataKey="enemyHP"
                stroke="#f87171"
                strokeWidth={2}
                dot={false}
                name="Enemy HP"
              />
            </LineChart>
          </ResponsiveContainer>
        </ChartContainer>

        {/* Chart 2: Time to Kill */}
        <ChartContainer title="2. Time to Kill (Ideal: 0.5-1.5s)">
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={data} margin={{ top: 10, right: 30, left: 10, bottom: 10 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#0f3460" />
              <XAxis
                dataKey="wave"
                stroke="#888"
                tick={{ fill: '#888', fontSize: 11 }}
              />
              <YAxis
                domain={[0, 3]}
                stroke="#888"
                tick={{ fill: '#888', fontSize: 11 }}
                tickFormatter={(value) => `${value}s`}
              />
              <Tooltip content={<CustomTooltip />} />
              <ReferenceLine y={0.5} stroke="#4ade80" strokeDasharray="5 5" />
              <ReferenceLine y={1.5} stroke="#4ade80" strokeDasharray="5 5" />
              <Line
                type="monotone"
                dataKey="ttk"
                stroke="#60a5fa"
                strokeWidth={2}
                dot={false}
                name="TTK"
              />
            </LineChart>
          </ResponsiveContainer>
          <div style={{ display: 'flex', justifyContent: 'center', gap: '24px', marginTop: '8px', fontSize: '11px' }}>
            <span style={{ color: '#4ade80' }}>-- Ideal Range (0.5s - 1.5s)</span>
          </div>
        </ChartContainer>

        {/* Chart 3: Spawn Pressure */}
        <ChartContainer title="3. Spawn Pressure (Green area = Kill Rate, Red line = Spawn Rate)">
          <ResponsiveContainer width="100%" height={200}>
            <AreaChart data={data} margin={{ top: 10, right: 30, left: 10, bottom: 10 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#0f3460" />
              <XAxis
                dataKey="wave"
                stroke="#888"
                tick={{ fill: '#888', fontSize: 11 }}
              />
              <YAxis
                stroke="#888"
                tick={{ fill: '#888', fontSize: 11 }}
                tickFormatter={(value) => `${value}/s`}
              />
              <Tooltip content={<CustomTooltip />} />
              <Legend wrapperStyle={{ fontSize: '12px' }} />
              <Area
                type="monotone"
                dataKey="killRate"
                stroke="#4ade80"
                fill="rgba(74, 222, 128, 0.3)"
                strokeWidth={2}
                name="Kill Rate"
              />
              <Line
                type="monotone"
                dataKey="spawnRate"
                stroke="#f87171"
                strokeWidth={2}
                dot={false}
                name="Spawn Rate"
              />
            </AreaChart>
          </ResponsiveContainer>
          <div style={{ textAlign: 'center', marginTop: '8px', fontSize: '11px', color: '#888' }}>
            When red line exceeds green area, player is being overwhelmed
          </div>
        </ChartContainer>

        {/* Chart 4: Active Enemies */}
        <ChartContainer title="4. Active Enemies at Equilibrium">
          <ResponsiveContainer width="100%" height={200}>
            <LineChart data={data} margin={{ top: 10, right: 30, left: 10, bottom: 10 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#0f3460" />
              <XAxis
                dataKey="wave"
                stroke="#888"
                tick={{ fill: '#888', fontSize: 11 }}
              />
              <YAxis
                stroke="#888"
                tick={{ fill: '#888', fontSize: 11 }}
              />
              <Tooltip content={<CustomTooltip />} />
              <ReferenceLine
                y={500}
                stroke="#fbbf24"
                strokeDasharray="5 5"
                label={{ value: 'Warning: 500', position: 'right', fill: '#fbbf24', fontSize: 11 }}
              />
              <Line
                type="monotone"
                dataKey="enemiesAlive"
                stroke="#a78bfa"
                strokeWidth={2}
                dot={false}
                name="Enemies Alive"
              />
            </LineChart>
          </ResponsiveContainer>
        </ChartContainer>

        {/* Formula Reference */}
        <div style={{
          backgroundColor: '#16213e',
          borderRadius: '12px',
          padding: '20px',
          fontSize: '12px',
          color: '#888'
        }}>
          <h4 style={{ color: '#e94560', marginBottom: '12px', fontSize: '14px' }}>Formula Reference</h4>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: '16px' }}>
            <div>
              <strong style={{ color: '#4ade80' }}>Player Power:</strong>
              <ul style={{ marginLeft: '16px', marginTop: '4px', lineHeight: '1.6' }}>
                <li>Kills to level: 25 × 1.2^(level-1)</li>
                <li>Creatures: ~0.6 per level</li>
                <li>Base DPS: 15 per creature</li>
                <li>Level bonus: +10% per creature level</li>
                <li>Crit bonus: 1.1x average</li>
                <li>Evolution: 2x at level 15+</li>
              </ul>
            </div>
            <div>
              <strong style={{ color: '#f87171' }}>Enemy Scaling:</strong>
              <ul style={{ marginLeft: '16px', marginTop: '4px', lineHeight: '1.6' }}>
                <li>Wave: every 50 kills</li>
                <li>HP: 30 × {enemyHpExponent.toFixed(2)}^wave</li>
                <li>Spawn: 0.8 + wave×0.1/s (max 4)</li>
                <li>Speed: 80 px/s</li>
                <li>Distance: 700px average</li>
              </ul>
            </div>
            <div>
              <strong style={{ color: '#60a5fa' }}>Derived Metrics:</strong>
              <ul style={{ marginLeft: '16px', marginTop: '4px', lineHeight: '1.6' }}>
                <li>TTK = Enemy HP / Player DPS</li>
                <li>Enemies = spawn × (travel + TTK)</li>
                <li>Kill Rate = DPS / HP</li>
                <li>Pressure = spawn / kill rate</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
