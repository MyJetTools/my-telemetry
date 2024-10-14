

## Tracking timer events

```rust


 async fn timer_tick(){
   let my_telemetry_tracker =
        my_telemetry::MyTelemetryContext::track_timer_duration("timer-name")
            .add_tag("client_id", client_id);


    call_function(parameter1, parameter2, &my_telemetry_tracker.my_telemetry);

 }
 


 fn call_function(parameter1: int64, parameter2: int64, my_telemetry: &MyTelemetry){
    // implement me
 }


```