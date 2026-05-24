use super::{
    DynInputPath, InteractionProfile, Left, MainAxisType, ProfileProperties, Property, Right,
    SkeletalInputBindings, legal_paths, paths::*,
};
use crate::button_mask_from_ids;
use crate::input::legacy::{self, LegacyBindings, button_mask_from_id};
use crate::input::profiles::InputToXrPath;
use crate::openxr_data::Hand;
use glam::{EulerRot, Mat4, Quat, Vec3};

pub struct MetaTouchPlus;

impl InteractionProfile for MetaTouchPlus {
    type LegalPaths = legal_paths![
        Both::<
            (Squeeze, Value),
            (Trigger, Value),
            (Trigger, Touch),
            (Thumbstick, ()),
            (Thumbstick, Click),
            (Thumbstick, Touch),
            (Thumbrest, Touch),
        >,
        Left::<(X, Click), (X, Touch), (Y, Click), (Y, Touch), (Menu, Click)>,
        Right::<(A, Click), (A, Touch), (B, Click), (B, Touch)>
    ];

    fn properties() -> &'static ProfileProperties {
        use openvr::EVRButtonId::*;
        static DEVICE_PROPERTIES: ProfileProperties = ProfileProperties {
            model: Property::PerHand {
                left: c"Meta Quest 3 (Left Controller)",
                right: c"Meta Quest 3 (Right Controller)",
            },
            openvr_controller_type: c"oculus_touch",
            render_model_name: Property::PerHand {
                left: c"meta_quest3_touch_plus_controller_left",
                right: c"meta_quest3_touch_plus_controller_right",
            },
            registered_device_type: Property::PerHand {
                left: c"oculus/WMHD315M3010GV_Controller_Left",
                right: c"oculus/WMHD315M3010GV_Controller_Right",
            },
            serial_number: Property::PerHand {
                left: c"WMHD315M3010GV_Controller_Left",
                right: c"WMHD315M3010GV_Controller_Right",
            },
            tracking_system_name: c"oculus",
            manufacturer_name: c"Oculus",
            main_axis: MainAxisType::Thumbstick,
            legacy_buttons_mask: button_mask_from_ids!(
                System,
                ApplicationMenu,
                Grip,
                A,
                Axis0,
                Axis1,
                Axis2
            ),
        };
        &DEVICE_PROPERTIES
    }

    fn profile_path() -> &'static str {
        "/interaction_profiles/meta/touch_controller_plus"
    }

    fn has_required_extensions(enabled_extensions: &openxr::ExtensionSet) -> bool {
        enabled_extensions.meta_touch_controller_plus
    }

    fn translate_path(path: DynInputPath) -> Option<DynInputPath> {
        match path {
            DynInputPath {
                subpath: DynSubpath::Trigger | DynSubpath::Squeeze,
                component: Some(DynComponent::Click),
                ..
            } => Some(DynInputPath {
                component: Some(DynComponent::Value),
                ..path
            }),
            _ => None,
        }
    }

    fn legacy_bindings(c: &InputToXrPath<Self>) -> LegacyBindings {
        LegacyBindings {
            extra: legacy::Bindings {
                grip_pose: c.pose(),
            },
            trigger: c.leftright::<Trigger, Value, _, _>(),
            trigger_click: c.leftright::<Trigger, Value, _, _>(),
            app_menu: [
                c.into::<Left<Y, Click>, _>(),
                c.into::<Right<B, Click>, _>(),
            ]
            .concat(),
            a: [
                c.into::<Left<X, Click>, _>(),
                c.into::<Right<A, Click>, _>(),
            ]
            .concat(),
            squeeze_click: c.leftright::<Squeeze, Value, _, _>(),
            squeeze: c.leftright::<Squeeze, Value, _, _>(),
            main_xy: c.leftright::<Thumbstick, (), _, _>(),
            main_xy_click: c.leftright::<Thumbstick, Click, _, _>(),
            main_xy_touch: c.leftright::<Thumbstick, Touch, _, _>(),
            haptic: c.haptics(),
        }
    }

    fn skeletal_input_bindings(c: &InputToXrPath<Self>) -> SkeletalInputBindings {
        SkeletalInputBindings {
            thumb_touch: [
                c.leftright::<Thumbstick, Touch, _, _>(),
                c.into::<Left<X, Touch>, _>(),
                c.into::<Left<Y, Touch>, _>(),
                c.into::<Right<A, Touch>, _>(),
                c.into::<Right<B, Touch>, _>(),
                c.leftright::<Thumbrest, Touch, _, _>(),
            ]
            .concat(),
            index_touch: c.leftright::<Trigger, Touch, _, _>(),
            index_curl: c.leftright::<Trigger, Value, _, _>(),
            rest_curl: c.leftright::<Squeeze, Value, _, _>(),
        }
    }

    fn offset_grip_pose(hand: Hand) -> Mat4 {
        match hand {
            Hand::Left => Mat4::from_rotation_translation(
                Quat::from_euler(
                    EulerRot::XYZ,
                    20.6_f32.to_radians(),
                    0.0_f32.to_radians(),
                    0.0_f32.to_radians(),
                ),
                Vec3::new(0.007, -0.00182941, 0.1019482),
            )
            .inverse(),
            Hand::Right => Mat4::from_rotation_translation(
                Quat::from_euler(
                    EulerRot::XYZ,
                    20.6_f32.to_radians(),
                    0.0_f32.to_radians(),
                    0.0_f32.to_radians(),
                ),
                Vec3::new(-0.007, -0.00182941, 0.1019482),
            )
            .inverse(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InteractionProfile, MetaTouchPlus};
    use crate::input::profiles::DynInputPath;
    use crate::input::tests::Fixture;
    use openxr as xr;

    #[test]
    fn clicky_paths_translate_to_value() {
        // The Plus profile only exposes analog trigger/squeeze, so a digital
        // binding to their `click` must fall back to the value component.
        let translate = |s: &str| {
            MetaTouchPlus::translate_path(s.parse::<DynInputPath>().unwrap()).map(|p| p.to_string())
        };
        assert_eq!(
            translate("/user/hand/left/input/trigger/click").as_deref(),
            Some("/user/hand/left/input/trigger/value")
        );
        // OpenVR names the squeeze subpath "grip"; it displays back as "squeeze".
        assert_eq!(
            translate("/user/hand/right/input/grip/click").as_deref(),
            Some("/user/hand/right/input/squeeze/value")
        );
        assert_eq!(translate("/user/hand/right/input/a/click"), None);
    }

    #[test]
    fn verify_bindings() {
        let f = Fixture::new();
        f.load_actions(c"actions.json");

        let path = MetaTouchPlus::profile_path();
        f.verify_bindings::<bool>(
            path,
            c"/actions/set1/in/boolact",
            [
                "/user/hand/left/input/x/click".into(),
                "/user/hand/left/input/y/click".into(),
                "/user/hand/right/input/a/click".into(),
                "/user/hand/right/input/b/click".into(),
                "/user/hand/right/input/thumbstick/click".into(),
                "/user/hand/right/input/thumbstick/touch".into(),
                "/user/hand/left/input/menu/click".into(),
            ],
        );

        f.verify_bindings::<f32>(
            path,
            c"/actions/set1/boolact_asfloat",
            [
                "/user/hand/left/input/squeeze/value".into(),
                "/user/hand/right/input/squeeze/value".into(),
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
            ],
        );

        f.verify_bindings::<f32>(
            path,
            c"/actions/set1/in/vec1act",
            [
                "/user/hand/left/input/trigger/value".into(),
                "/user/hand/right/input/trigger/value".into(),
            ],
        );

        f.verify_bindings::<xr::Vector2f>(
            path,
            c"/actions/set1/in/vec2act",
            [
                "/user/hand/left/input/thumbstick".into(),
                "/user/hand/right/input/thumbstick".into(),
            ],
        );

        f.verify_bindings::<xr::Haptic>(
            path,
            c"/actions/set1/in/vib",
            [
                "/user/hand/left/output/haptic".into(),
                "/user/hand/right/output/haptic".into(),
            ],
        );
    }
}
