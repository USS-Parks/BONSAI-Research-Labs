# Primary behavior metrics

BK-02 derives cumulative reward, reward rate, comparator-qualified regret, lifelong performance AUC, first competency time, mean/worst recovery after declared change points, worst rolling-window reward rate, and exact-age reward rates. Every result stores its version, unit, and window.

The engine uses checked normalized rational arithmetic over a contiguous step trace. Regret is unavailable unless every point supplies the declared comparator. Recovery is unavailable when any change point never regains competency. These states are evidence gaps, never numeric zero.
