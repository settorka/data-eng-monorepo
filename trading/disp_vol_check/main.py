import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from arch import arch_model
import yfinance as yf
from scipy import stats
import warnings
warnings.filterwarnings('ignore')

# Set style
plt.style.use('seaborn-v0_8-darkgrid')
sns.set_palette("husl")

# ============================================
# 1. GET REAL APPLE OHLCV DATA
# ============================================
print("Fetching APPLE OHLCV data...")
ticker = "AAPL"

# Get 2 years of daily data - FIXED VERSION
APPLE = yf.download(ticker, start="2022-01-01", end="2024-01-01", progress=False)

# Check if columns are MultiIndex and flatten if needed
if isinstance(APPLE.columns, pd.MultiIndex):
    APPLE.columns = APPLE.columns.get_level_values(0)

print(f"Data shape: {APPLE.shape}")
print(f"Date range: {APPLE.index[0].date()} to {APPLE.index[-1].date()}")
print("\nFirst few rows:")
print(APPLE[['Open', 'High', 'Low', 'Close', 'Volume']].head())

# ============================================
# 2. COMPUTE RETURNS FROM CLOSE PRICES
# ============================================
# Log returns are better for volatility modeling
APPLE['Log_Returns'] = np.log(APPLE['Close'] / APPLE['Close'].shift(1))
APPLE = APPLE.dropna()

# Also compute Parkinson volatility (High-Low estimator) for comparison
APPLE['HL_Range'] = np.log(APPLE['High'] / APPLE['Low'])
APPLE['Parkinson_Vol'] = (1/(4*np.log(2))) * (APPLE['HL_Range']**2)

print(f"\n=== RETURN STATISTICS ===")
print(f"Mean return: {APPLE['Log_Returns'].mean():.6f}")
print(f"Std dev: {APPLE['Log_Returns'].std():.6f}")
print(f"Skewness: {APPLE['Log_Returns'].skew():.4f}")
print(f"Kurtosis: {APPLE['Log_Returns'].kurtosis():.4f}")
print(f"Min return: {APPLE['Log_Returns'].min():.6f}")
print(f"Max return: {APPLE['Log_Returns'].max():.6f}")

# ============================================
# 3. GARCH MODELING
# ============================================
print("\n" + "="*50)
print("GARCH(1,1) MODEL FITTING")
print("="*50)

# Fit GARCH(1,1) model
garch = arch_model(APPLE['Log_Returns'] * 100, vol='Garch', p=1, q=1, dist='normal')
garch_fit = garch.fit(update_freq=5, disp='off')

print(garch_fit.summary())

# Get conditional volatility from GARCH
APPLE['GARCH_Volatility'] = garch_fit.conditional_volatility / 100  # Convert back from percentage

# ============================================
# 4. EGARCH MODELING (with better settings)
# ============================================
print("\n" + "="*50)
print("EGARCH(1,1) MODEL FITTING")
print("="*50)

# Try with different distributions and optimizer settings
distributions = ['normal', 'ged', 't']
egarch_success = False

for dist in distributions:
    try:
        print(f"Trying EGARCH with {dist.upper()} distribution...")
        egarch = arch_model(APPLE['Log_Returns'] * 100, vol='EGARCH', p=1, q=1, o=1, dist=dist)
        
        # Use more robust optimization
        egarch_fit = egarch.fit(update_freq=5, disp='off', show_warning=False,
                               options={'maxiter': 1000, 'ftol': 1e-10})
        
        print(f"Success with {dist.upper()} distribution!")
        print(egarch_fit.summary())
        
        # Get conditional volatility from EGARCH
        APPLE['EGARCH_Volatility'] = egarch_fit.conditional_volatility / 100
        egarch_success = True
        break
        
    except Exception as e:
        print(f"  {dist.upper()} failed: {str(e)[:80]}...")
        continue

if not egarch_success:
    print("\nAll EGARCH specifications failed. Using GARCH volatility as proxy.")
    APPLE['EGARCH_Volatility'] = APPLE['GARCH_Volatility']

# ============================================
# 5. DISPERSION ANALYSIS
# ============================================
print("\n" + "="*50)
print("VOLATILITY DISPERSION ANALYSIS")
print("="*50)

# Compute rolling statistics of volatility
window = 20  # 1 month trading window

# Rolling statistics for GARCH volatility
APPLE['GARCH_Vol_MA'] = APPLE['GARCH_Volatility'].rolling(window=window).mean()
APPLE['GARCH_Vol_Std'] = APPLE['GARCH_Volatility'].rolling(window=window).std()
APPLE['GARCH_Vol_Skew'] = APPLE['GARCH_Volatility'].rolling(window=window).apply(lambda x: stats.skew(x))
APPLE['GARCH_Vol_Dispersion'] = APPLE['GARCH_Vol_Std'] / APPLE['GARCH_Vol_MA']  # Coefficient of variation

# Same for EGARCH
APPLE['EGARCH_Vol_Std'] = APPLE['EGARCH_Volatility'].rolling(window=window).std()
APPLE['EGARCH_Vol_Dispersion'] = APPLE['EGARCH_Vol_Std'] / APPLE['EGARCH_Volatility'].rolling(window=window).mean()

print("\nOverall Dispersion Statistics:")
print(f"GARCH Volatility Mean: {APPLE['GARCH_Volatility'].mean():.6f}")
print(f"GARCH Volatility Std Dev: {APPLE['GARCH_Volatility'].std():.6f}")
print(f"GARCH Volatility CV (Dispersion): {APPLE['GARCH_Volatility'].std()/APPLE['GARCH_Volatility'].mean():.4f}")
print(f"EGARCH Volatility CV (Dispersion): {APPLE['EGARCH_Volatility'].std()/APPLE['EGARCH_Volatility'].mean():.4f}")

# ============================================
# 6. REGIME DETECTION (High/Low Volatility)
# ============================================
# Identify volatility regimes
vol_threshold = APPLE['GARCH_Volatility'].quantile(0.75)  # Top 25% as high vol
APPLE['Vol_Regime'] = np.where(APPLE['GARCH_Volatility'] > vol_threshold, 'High Vol', 'Low Vol')

# Statistics by regime
regime_stats = APPLE.groupby('Vol_Regime')['Log_Returns'].agg(['mean', 'std', 'count'])
print("\n=== RETURN STATISTICS BY VOLATILITY REGIME ===")
print(regime_stats)

# ============================================
# 7. VISUALIZATION
# ============================================
fig, axes = plt.subplots(4, 2, figsize=(15, 18))
fig.suptitle(f'APPLE ({ticker}) Volatility Analysis with GARCH/EGARCH Models', fontsize=16, y=1.02)

# Plot 1: Price and Returns
axes[0, 0].plot(APPLE.index, APPLE['Close'], color='blue', linewidth=1.5)
axes[0, 0].set_title('APPLE Stock Price', fontsize=12)
axes[0, 0].set_ylabel('Price ($)')
axes[0, 0].grid(True, alpha=0.3)

axes[0, 1].plot(APPLE.index, APPLE['Log_Returns'], color='red', linewidth=0.8, alpha=0.7)
axes[0, 1].axhline(y=0, color='black', linestyle='-', linewidth=0.5)
axes[0, 1].set_title('Daily Log Returns', fontsize=12)
axes[0, 1].set_ylabel('Returns')
axes[0, 1].grid(True, alpha=0.3)

# Plot 2: GARCH vs EGARCH Volatility
axes[1, 0].plot(APPLE.index, APPLE['GARCH_Volatility'], color='green', linewidth=1.5, alpha=0.8, label='GARCH(1,1)')
axes[1, 0].fill_between(APPLE.index, 0, APPLE['GARCH_Volatility'], alpha=0.2, color='green')
axes[1, 0].set_title('GARCH(1,1) Conditional Volatility', fontsize=12)
axes[1, 0].set_ylabel('Volatility')
axes[1, 0].legend()
axes[1, 0].grid(True, alpha=0.3)

axes[1, 1].plot(APPLE.index, APPLE['EGARCH_Volatility'], color='purple', linewidth=1.5, alpha=0.8, label='EGARCH(1,1)')
axes[1, 1].fill_between(APPLE.index, 0, APPLE['EGARCH_Volatility'], alpha=0.2, color='purple')
axes[1, 1].set_title('EGARCH(1,1) Conditional Volatility', fontsize=12)
axes[1, 1].set_ylabel('Volatility')
axes[1, 1].legend()
axes[1, 1].grid(True, alpha=0.3)

# Plot 3: Volatility Dispersion
axes[2, 0].plot(APPLE.index, APPLE['GARCH_Vol_Dispersion'], color='orange', linewidth=1.5)
axes[2, 0].axhline(y=APPLE['GARCH_Vol_Dispersion'].mean(), color='red', linestyle='--', label=f'Mean: {APPLE["GARCH_Vol_Dispersion"].mean():.3f}')
axes[2, 0].set_title('GARCH Volatility Dispersion (Coefficient of Variation)', fontsize=12)
axes[2, 0].set_ylabel('Dispersion')
axes[2, 0].legend()
axes[2, 0].grid(True, alpha=0.3)

# Plot 4: Rolling Volatility Statistics
axes[2, 1].plot(APPLE.index, APPLE['GARCH_Vol_MA'], color='blue', label='20-day MA', linewidth=1.5)
axes[2, 1].plot(APPLE.index, APPLE['GARCH_Volatility'], color='gray', alpha=0.3, label='Daily Vol', linewidth=0.5)
axes[2, 1].fill_between(APPLE.index, 
                        APPLE['GARCH_Vol_MA'] - APPLE['GARCH_Vol_Std'], 
                        APPLE['GARCH_Vol_MA'] + APPLE['GARCH_Vol_Std'], 
                        alpha=0.2, color='blue', label='±1 Std Dev')
axes[2, 1].set_title('Rolling Volatility with Dispersion Bands', fontsize=12)
axes[2, 1].set_ylabel('Volatility')
axes[2, 1].legend()
axes[2, 1].grid(True, alpha=0.3)

# Plot 5: Volatility Distribution
axes[3, 0].hist(APPLE['GARCH_Volatility'].dropna(), bins=50, color='green', alpha=0.7, edgecolor='black', density=True)
axes[3, 0].axvline(x=APPLE['GARCH_Volatility'].mean(), color='red', linestyle='--', 
                   label=f'Mean: {APPLE["GARCH_Volatility"].mean():.4f}')
axes[3, 0].set_title('Distribution of GARCH Volatility', fontsize=12)
axes[3, 0].set_xlabel('Volatility')
axes[3, 0].set_ylabel('Density')
axes[3, 0].legend()
axes[3, 0].grid(True, alpha=0.3)

# Plot 6: Volume vs Volatility
scatter = axes[3, 1].scatter(APPLE['Volume']/1e6, APPLE['GARCH_Volatility']*100, 
                            c=APPLE['Log_Returns'], cmap='coolwarm', alpha=0.6, s=20)
axes[3, 1].set_title('Trading Volume vs. Volatility (color = return)', fontsize=12)
axes[3, 1].set_xlabel('Volume (millions)')
axes[3, 1].set_ylabel('Volatility (%)')
plt.colorbar(scatter, ax=axes[3, 1], label='Return')

plt.tight_layout()
plt.show()

# ============================================
# 8. KEY INSIGHTS SUMMARY
# ============================================
print("\n" + "="*50)
print("KEY INSIGHTS SUMMARY")
print("="*50)

print("\n1. VOLATILITY CHARACTERISTICS:")
print(f"   - Average daily volatility: {APPLE['GARCH_Volatility'].mean()*100:.2f}%")
print(f"   - Maximum volatility: {APPLE['GARCH_Volatility'].max()*100:.2f}%")
print(f"   - Minimum volatility: {APPLE['GARCH_Volatility'].min()*100:.2f}%")

print("\n2. DISPERSION ANALYSIS:")
print(f"   - Volatility of volatility: {APPLE['GARCH_Volatility'].std()*100:.3f}%")
print(f"   - Coefficient of Variation: {APPLE['GARCH_Volatility'].std()/APPLE['GARCH_Volatility'].mean():.3f}")
print(f"   - Higher CV means more erratic/unpredictable volatility")

print("\n3. REGIME ANALYSIS:")
print(f"   - High volatility days (>75th percentile): {regime_stats.loc['High Vol', 'count']}")
print(f"   - Low volatility days: {regime_stats.loc['Low Vol', 'count']}")
print(f"   - Returns in high vol periods: {regime_stats.loc['High Vol', 'mean']*100:.3f}%")
print(f"   - Returns in low vol periods: {regime_stats.loc['Low Vol', 'mean']*100:.3f}%")

print("\n4. EGARCH LEVERAGE EFFECT:")
egarch_params = egarch_fit.params
if egarch_params.get('gamma[1]', 0) < 0:
    print(f"   - Gamma parameter: {egarch_params.get('gamma[1]', 0):.4f}")
    print("   - NEGATIVE gamma indicates leverage effect (bad news increases vol more than good news)")
else:
    print("   - No significant leverage effect detected")

print("\n5. TRADING IMPLICATIONS:")
print("   - High dispersion periods: Options more expensive, hedging more challenging")
print("   - Low dispersion periods: More predictable risk environment")
print("   - Use EGARCH for options pricing (captures asymmetry)")
print("   - Use GARCH for risk management (VaR calculations)")

# Save results to CSV
APPLE[['Close', 'Log_Returns', 'GARCH_Volatility', 'EGARCH_Volatility', 
       'GARCH_Vol_Dispersion', 'Vol_Regime']].to_csv(r'trading/disp_vol_check/APPLE_volatility_analysis.csv')
print(f"\nResults saved to 'APPLE_volatility_analysis.csv'")