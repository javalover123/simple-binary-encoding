use std::fs::File;
use std::io::prelude::*;

use examples_extension::*;

fn read_sbe_file_generated_from_java_example() -> ::std::io::Result<Vec<u8>> {
    // Generated by the generateCarExampleDataFile gradle task.
    let mut f = File::open("car_example_extension_data.sbe")?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[test]
fn run_car_extension_example() -> SbeResult<()> {
    match read_sbe_file_generated_from_java_example() {
        Ok(reference_example_bytes) => {
            decode_car_and_assert_expected_content(&reference_example_bytes)?;
            let (limit, mut bytes_encoded_from_rust) = encode_car_from_scratch()?;
            decode_car_and_assert_expected_content(bytes_encoded_from_rust.as_slice())?;
            bytes_encoded_from_rust.truncate(limit);
        }
        Err(e) => {
            panic!("{:?}", e)
        }
    }
    Ok(())
}

fn decode_car_and_assert_expected_content(buffer: &[u8]) -> SbeResult<()> {
    let mut car = CarDecoder::default();

    let buf = ReadBuf::new(buffer);
    let header = MessageHeaderDecoder::default().wrap(buf, 0);
    assert_eq!(car::SBE_TEMPLATE_ID, header.template_id());
    car = car.header(header);

    // Car...
    assert_eq!(1234, car.serial_number());
    assert_eq!(2013, car.model_year());
    assert_eq!(BooleanType::T, car.available());

    assert_eq!(Model::A, car.code());

    assert_eq!([0, 1, 2, 3], car.some_numbers());
    assert_eq!([97, 98, 99, 100, 101, 102], car.vehicle_code());

    let extras = car.extras();
    assert_eq!(6, extras.0);
    assert!(extras.get_cruise_control());
    assert!(extras.get_sports_pack());
    assert!(!extras.get_sun_roof());

    assert_eq!(Model::C, car.discounted_model());

    let mut engine = car.engine_decoder();
    assert_eq!(2000, engine.capacity());
    assert_eq!(4, engine.num_cylinders());
    assert_eq!(9000, engine.max_rpm());
    assert_eq!("123", String::from_utf8_lossy(&engine.manufacturer_code()));

    assert_eq!(b"Petrol", engine.fuel());
    assert_eq!(35, engine.efficiency());
    assert_eq!(BooleanType::T, engine.booster_enabled());

    let mut booster = engine.booster_decoder();
    assert_eq!(BoostType::NITROUS, booster.boost_type());
    assert_eq!(200, booster.horse_power());

    engine = booster.parent()?;
    car = engine.parent()?;
    let mut fuel_figures = car.fuel_figures_decoder();
    assert_eq!(3, fuel_figures.count());

    assert_eq!(Some(0), fuel_figures.advance()?);
    assert_eq!(30, fuel_figures.speed());
    assert_eq!(35.9, fuel_figures.mpg());
    let coord = fuel_figures.usage_description_decoder();
    assert_eq!(
        "Urban Cycle",
        String::from_utf8_lossy(fuel_figures.usage_description_slice(coord))
    );

    assert_eq!(Some(1), fuel_figures.advance()?);
    assert_eq!(55, fuel_figures.speed());
    assert_eq!(49.0, fuel_figures.mpg());
    let coord = fuel_figures.usage_description_decoder();
    assert_eq!(
        "Combined Cycle",
        String::from_utf8_lossy(fuel_figures.usage_description_slice(coord))
    );

    assert_eq!(Some(2), fuel_figures.advance()?);
    assert_eq!(75, fuel_figures.speed());
    assert_eq!(40.0, fuel_figures.mpg());
    let coord = fuel_figures.usage_description_decoder();
    assert_eq!(
        "Highway Cycle",
        String::from_utf8_lossy(fuel_figures.usage_description_slice(coord))
    );
    assert_eq!(Ok(None), fuel_figures.advance());

    car = fuel_figures.parent()?;
    let mut performance_figures = car.performance_figures_decoder();
    assert_eq!(2, performance_figures.count());

    // 95 octane
    assert_eq!(Some(0), performance_figures.advance()?);
    assert_eq!(95, performance_figures.octane_rating());
    let mut acceleration = performance_figures.acceleration_decoder();
    assert_eq!(3, acceleration.count());
    assert_eq!(Some(0), acceleration.advance()?);
    assert_eq!(30, acceleration.mph());
    assert_eq!(4.0, acceleration.seconds());
    assert_eq!(Some(1), acceleration.advance()?);
    assert_eq!(60, acceleration.mph());
    assert_eq!(7.5, acceleration.seconds());
    assert_eq!(Some(2), acceleration.advance()?);
    assert_eq!(100, acceleration.mph());
    assert_eq!(12.2, acceleration.seconds());
    assert_eq!(Ok(None), acceleration.advance());

    // 99 octane
    performance_figures = acceleration.parent()?;
    assert_eq!(Some(1), performance_figures.advance()?);
    assert_eq!(99, performance_figures.octane_rating());
    acceleration = performance_figures.acceleration_decoder();
    assert_eq!(3, acceleration.count());
    assert_eq!(Some(0), acceleration.advance()?);
    assert_eq!(30, acceleration.mph());
    assert_eq!(3.8, acceleration.seconds());
    assert_eq!(Some(1), acceleration.advance()?);
    assert_eq!(60, acceleration.mph());
    assert_eq!(7.1, acceleration.seconds());
    assert_eq!(Some(2), acceleration.advance()?);
    assert_eq!(100, acceleration.mph());
    assert_eq!(11.8, acceleration.seconds());
    assert_eq!(Ok(None), acceleration.advance());

    performance_figures = acceleration.parent()?;
    car = performance_figures.parent()?;

    let coord = car.manufacturer_decoder();
    assert_eq!(
        "Honda",
        String::from_utf8_lossy(car.manufacturer_slice(coord))
    );

    let coord = car.model_decoder();
    assert_eq!("Civic VTi", String::from_utf8_lossy(car.model_slice(coord)));

    let coord = car.activation_code_decoder();
    assert_eq!(
        "abcdef",
        String::from_utf8_lossy(car.activation_code_slice(coord))
    );

    Ok(())
}

fn encode_car_from_scratch() -> SbeResult<(usize, Vec<u8>)> {
    let mut buffer = vec![0u8; 256];
    let mut car = CarEncoder::default();
    let mut engine = EngineEncoder::default();
    let mut booster = BoosterEncoder::default();
    let mut fuel_figures = FuelFiguresEncoder::default();
    let mut performance_figures = PerformanceFiguresEncoder::default();
    let mut acceleration = AccelerationEncoder::default();
    let mut extras = OptionalExtras::default();

    car = car.wrap(
        WriteBuf::new(buffer.as_mut_slice()),
        message_header::ENCODED_LENGTH,
    );
    car = car.header(0).parent()?;

    car.serial_number(1234);
    car.model_year(2013);
    car.available(BooleanType::T);
    car.code(Model::A);
    car.some_numbers([0, 1, 2, 3]);
    car.vehicle_code([97, 98, 99, 100, 101, 102]); // abcdef

    extras.set_cruise_control(true);
    extras.set_sports_pack(true);
    extras.set_sun_roof(false);
    car.extras(extras);

    engine = car.engine_encoder(engine);
    engine.capacity(2000);
    engine.num_cylinders(4);
    engine.manufacturer_code([49, 50, 51]); // 123
    engine.efficiency(35);
    engine.booster_enabled(BooleanType::T);
    booster = engine.booster_encoder(booster);
    booster.boost_type(BoostType::NITROUS);
    booster.horse_power(200);

    engine = booster.parent()?;
    car = engine.parent()?;
    fuel_figures = car.fuel_figures_encoder(3, fuel_figures);
    assert_eq!(Some(0), fuel_figures.advance()?);
    fuel_figures.speed(30);
    fuel_figures.mpg(35.9);
    fuel_figures.usage_description(b"Urban Cycle");

    assert_eq!(Some(1), fuel_figures.advance()?);
    fuel_figures.speed(55);
    fuel_figures.mpg(49.0);
    fuel_figures.usage_description(b"Combined Cycle");

    assert_eq!(Some(2), fuel_figures.advance()?);
    fuel_figures.speed(75);
    fuel_figures.mpg(40.0);
    fuel_figures.usage_description(b"Highway Cycle");

    car = fuel_figures.parent()?;
    performance_figures = car.performance_figures_encoder(2, performance_figures);
    assert_eq!(Some(0), performance_figures.advance()?);
    performance_figures.octane_rating(95);

    acceleration = performance_figures.acceleration_encoder(3, acceleration);
    assert_eq!(Some(0), acceleration.advance()?);
    acceleration.mph(30);
    acceleration.seconds(4.0);

    assert_eq!(Some(1), acceleration.advance()?);
    acceleration.mph(60);
    acceleration.seconds(7.5);

    assert_eq!(Some(2), acceleration.advance()?);
    acceleration.mph(100);
    acceleration.seconds(12.2);

    performance_figures = acceleration.parent()?;
    assert_eq!(Some(1), performance_figures.advance()?);
    performance_figures.octane_rating(99);

    acceleration = performance_figures.acceleration_encoder(3, acceleration);
    assert_eq!(Some(0), acceleration.advance()?);
    acceleration.mph(30);
    acceleration.seconds(3.8);

    assert_eq!(Some(1), acceleration.advance()?);
    acceleration.mph(60);
    acceleration.seconds(7.1);

    assert_eq!(Some(2), acceleration.advance()?);
    acceleration.mph(100);
    acceleration.seconds(11.8);

    performance_figures = acceleration.parent()?;
    car = performance_figures.parent()?;

    car.manufacturer("Honda");
    car.model("Civic VTi");
    car.activation_code(b"abcdef");

    let limit = car.get_limit();
    Ok((limit, buffer))
}
