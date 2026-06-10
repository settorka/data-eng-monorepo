import numpy as np
import pandas as pd
from arch import arch_model
import yfinance as yf
import warnings
warnings.filterwarnings('ignore')

# ============================================
# CORE ANALYSIS FUNCTIONS
# ============================================
def get_stock_data(ticker, start_date="2022-01-01", end_date="2024-01-01"):
    """Download and clean stock data"""
    data = yf.download(ticker, start=start_date, end=end_date, progress=False)
    
    if isinstance(data.columns, pd.MultiIndex):
        data.columns = data.columns.get_level_values(0)
    
    return data


def calculate_returns(data):
    """Calculate log returns and basic stats"""
    data = data.copy()
    data['Log_Returns'] = np.log(data['Close'] / data['Close'].shift(1))
    data = data.dropna()
    
    stats = {
        'mean': data['Log_Returns'].mean(),
        'std': data['Log_Returns'].std(),
        'skew': data['Log_Returns'].skew(),
        'kurt': data['Log_Returns'].kurtosis(),
        'min': data['Log_Returns'].min(),
        'max': data['Log_Returns'].max()
    }
    
    return data, stats


def fit_garch_model(returns_series):
    """Fit GARCH(1,1) model"""
    garch = arch_model(returns_series * 100, vol='Garch', p=1, q=1, dist='normal')
    garch_fit = garch.fit(update_freq=0, disp='off')
    
    # Get parameters
    params = garch_fit.params
    garch_results = {
        'omega': params.get('omega', 0),
        'alpha': params.get('alpha[1]', 0),
        'beta': params.get('beta[1]', 0),
        'persistence': params.get('alpha[1]', 0) + params.get('beta[1]', 0),
        'conditional_vol': garch_fit.conditional_volatility / 100
    }
    
    return garch_fit, garch_results


def fit_egarch_model(returns_series):
    """Fit EGARCH(1,1) model with fallback"""
    try:
        egarch = arch_model(returns_series * 100, vol='EGARCH', p=1, q=1, o=1, dist='normal')
        egarch_fit = egarch.fit(update_freq=0, disp='off', show_warning=False)
        
        params = egarch_fit.params
        egarch_results = {
            'omega': params.get('omega', 0),
            'alpha': params.get('alpha[1]', 0),
            'gamma': params.get('gamma[1]', 0),
            'beta': params.get('beta[1]', 0),
            'conditional_vol': egarch_fit.conditional_volatility / 100
        }
        
        return egarch_fit, egarch_results
    except:
        return None, None


def analyze_dispersion(volatility_series):
    """Calculate dispersion (coefficient of variation)"""
    if len(volatility_series) == 0:
        return 0.0
    return volatility_series.std() / volatility_series.mean()


def analyze_volatility_regimes(returns_series, volatility_series):
    """Analyze returns in high vs low volatility periods"""
    # Align indices
    returns_aligned = returns_series[volatility_series.index]
    
    # Define high volatility (top 25%)
    threshold = volatility_series.quantile(0.75)
    high_vol_mask = volatility_series > threshold
    
    high_vol_returns = returns_aligned[high_vol_mask].mean() if high_vol_mask.any() else 0.0
    low_vol_returns = returns_aligned[~high_vol_mask].mean() if (~high_vol_mask).any() else 0.0
    
    return {
        'high_vol_return': high_vol_returns,
        'low_vol_return': low_vol_returns,
        'high_vol_days': int(high_vol_mask.sum()),
        'low_vol_days': int((~high_vol_mask).sum())
    }


def analyze_stock(ticker, start_date="2022-01-01", end_date="2024-01-01"):
    """Complete analysis for a single stock"""
    print(f"\n{'='*60}")
    print(f"ANALYZING {ticker}")
    print(f"{'='*60}")
    
    # 1. Get data
    print("Fetching data...")
    data = get_stock_data(ticker, start_date, end_date)
    
    if data.empty:
        print(f"No data for {ticker}")
        return None
    
    print(f"Data shape: {data.shape}")
    print(f"Date range: {data.index[0].date()} to {data.index[-1].date()}")
    
    # 2. Calculate returns
    data, return_stats = calculate_returns(data)
    print(f"\nReturn Statistics:")
    print(f"  Mean return: {return_stats['mean']:.6f}")
    print(f"  Std dev: {return_stats['std']:.6f} ({return_stats['std']*100:.2f}% daily)")
    print(f"  Skewness: {return_stats['skew']:.4f}")
    print(f"  Kurtosis: {return_stats['kurt']:.4f}")
    
    # 3. Fit GARCH model
    print("\nFitting GARCH(1,1)...")
    garch_fit, garch_results = fit_garch_model(data['Log_Returns'])
    
    print(f"GARCH Parameters:")
    print(f"  Alpha (news impact): {garch_results['alpha']:.4f}")
    print(f"  Beta (persistence): {garch_results['beta']:.4f}")
    print(f"  Persistence (α+β): {garch_results['persistence']:.4f}")
    
    # Add GARCH volatility to data
    data['GARCH_Volatility'] = pd.Series(garch_results['conditional_vol'], index=data.index)
    
    # 4. Fit EGARCH model
    print("\nFitting EGARCH(1,1)...")
    egarch_fit, egarch_results = fit_egarch_model(data['Log_Returns'])
    
    if egarch_results:
        print(f"EGARCH Parameters:")
        print(f"  Gamma (leverage effect): {egarch_results['gamma']:.4f}")
        data['EGARCH_Volatility'] = pd.Series(egarch_results['conditional_vol'], index=data.index)
    else:
        print("EGARCH failed, using GARCH as proxy")
        data['EGARCH_Volatility'] = data['GARCH_Volatility']
        egarch_results = {'gamma': None}
    
    # 5. Calculate dispersion
    garch_dispersion = analyze_dispersion(data['GARCH_Volatility'])
    egarch_dispersion = analyze_dispersion(data['EGARCH_Volatility'])
    
    print(f"\nVolatility Dispersion:")
    print(f"  GARCH dispersion (CV): {garch_dispersion:.4f}")
    print(f"  EGARCH dispersion (CV): {egarch_dispersion:.4f}")
    
    # 6. Analyze volatility regimes
    regime_stats = analyze_volatility_regimes(data['Log_Returns'], data['GARCH_Volatility'])
    
    print(f"\nVolatility Regime Analysis:")
    print(f"  High volatility days: {regime_stats['high_vol_days']}")
    print(f"  Low volatility days: {regime_stats['low_vol_days']}")
    print(f"  Return during high vol: {regime_stats['high_vol_return']*100:.3f}%")
    print(f"  Return during low vol: {regime_stats['low_vol_return']*100:.3f}%")
    
    # 7. Determine trading pattern
    if regime_stats['high_vol_return'] < regime_stats['low_vol_return']:
        pattern = "Buy when boring (sells off in high vol)"
    else:
        pattern = "Buy when exciting (rallies in high vol)"
    
    print(f"\nTrading Pattern: {pattern}")
    
    # 8. Check leverage effect
    if egarch_results['gamma'] is not None:
        if egarch_results['gamma'] < 0:
            print("Leverage Effect: YES (bad news increases vol more than good news)")
        else:
            print("Leverage Effect: NO or positive")
    
    # Compile all results
    results = {
        'ticker': ticker,
        'return_stats': return_stats,
        'garch_results': garch_results,
        'egarch_results': egarch_results,
        'garch_dispersion': garch_dispersion,
        'egarch_dispersion': egarch_dispersion,
        'regime_stats': regime_stats,
        'pattern': pattern,
        'data': data[['Close', 'Log_Returns', 'GARCH_Volatility', 'EGARCH_Volatility']]
    }
    
    return results


def analyze_stock_list(ticker_list, start_date="2022-01-01", end_date="2024-01-01"):
    """Analyze multiple stocks"""
    all_results = {}
    
    for ticker in ticker_list:
        print(f"\n{'#'*70}")
        print(f"PROCESSING: {ticker}")
        print(f"{'#'*70}")
        
        try:
            results = analyze_stock(ticker, start_date, end_date)
            if results:
                all_results[ticker] = results
        except Exception as e:
            print(f"Error analyzing {ticker}: {e}")
            continue
    
    return all_results


def print_summary_table(results_dict):
    """Print a clean summary table of all stocks"""
    print(f"\n{'='*80}")
    print("SUMMARY TABLE")
    print(f"{'='*80}")
    print(f"{'Ticker':<8} {'Volatility':<12} {'High Vol Return':<18} {'Low Vol Return':<18} {'Pattern':<30} {'Dispersion':<12}")
    print(f"{'-'*80}")
    
    for ticker, results in results_dict.items():
        vol = results['return_stats']['std'] * 100
        high_vol = results['regime_stats']['high_vol_return'] * 100
        low_vol = results['regime_stats']['low_vol_return'] * 100
        pattern = results['pattern'][:28]  # Truncate if too long
        dispersion = results['garch_dispersion']
        
        print(f"{ticker:<8} {vol:>6.2f}%{'':<4} {high_vol:>7.3f}%{'':<8} {low_vol:>7.3f}%{'':<8} {pattern:<30} {dispersion:>7.3f}")


def save_results_to_csv(results_dict, filename="stock_volatility_results.csv"):
    """Save all results to CSV"""
    summary_data = []
    
    for ticker, results in results_dict.items():
        summary_data.append({
            'Ticker': ticker,
            'Start_Date': results['data'].index[0].date(),
            'End_Date': results['data'].index[-1].date(),
            'Mean_Return': results['return_stats']['mean'],
            'Daily_Volatility': results['return_stats']['std'],
            'Annualized_Vol': results['return_stats']['std'] * np.sqrt(252),
            'Skewness': results['return_stats']['skew'],
            'Kurtosis': results['return_stats']['kurt'],
            'High_Vol_Return': results['regime_stats']['high_vol_return'],
            'Low_Vol_Return': results['regime_stats']['low_vol_return'],
            'High_Vol_Days': results['regime_stats']['high_vol_days'],
            'Low_Vol_Days': results['regime_stats']['low_vol_days'],
            'Trading_Pattern': results['pattern'],
            'GARCH_Dispersion': results['garch_dispersion'],
            'EGARCH_Dispersion': results['egarch_dispersion'],
            'GARCH_Alpha': results['garch_results']['alpha'],
            'GARCH_Beta': results['garch_results']['beta'],
            'GARCH_Persistence': results['garch_results']['persistence'],
            'EGARCH_Gamma': results['egarch_results'].get('gamma', None),
            'Leverage_Effect': 'Yes' if results['egarch_results'].get('gamma', 0) < 0 else 'No/Unknown'
        })
    
    df = pd.DataFrame(summary_data)
    df.to_csv(filename, index=False)
    print(f"\nResults saved to {filename}")
    
    return df


# ============================================
# MAIN EXECUTION
# ============================================
if __name__ == "__main__":
    # Define your stock list
    stocks = ["AAPL", "MSFT", "TSLA", "NVDA", "JNJ", "PG", "KO", "AMD"]
    
    # Analyze all stocks
    results = analyze_stock_list(stocks)
    
    # Print summary table
    print_summary_table(results)
    
    # Save results
    if results:
        save_results_to_csv(results)
        
        # Print some insights
        print(f"\n{'='*80}")
        print("KEY INSIGHTS")
        print(f"{'='*80}")
        
        # Count patterns
        boring_count = sum(1 for r in results.values() if "Buy when boring" in r['pattern'])
        exciting_count = sum(1 for r in results.values() if "Buy when exciting" in r['pattern'])
        
        print(f"Stocks that 'Buy when boring': {boring_count}")
        print(f"Stocks that 'Buy when exciting': {exciting_count}")
        
        # Find highest/lowest volatility
        if results:
            highest_vol = max(results.items(), key=lambda x: x[1]['return_stats']['std'])
            lowest_vol = min(results.items(), key=lambda x: x[1]['return_stats']['std'])
            
            print(f"\nHighest volatility: {highest_vol[0]} ({highest_vol[1]['return_stats']['std']*100:.2f}%)")
            print(f"Lowest volatility: {lowest_vol[0]} ({lowest_vol[1]['return_stats']['std']*100:.2f}%)")