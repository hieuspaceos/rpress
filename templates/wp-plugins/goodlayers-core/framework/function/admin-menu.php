<?php
	/*	
	*	Goodlayers Admin Menu
	*	---------------------------------------------------------------------
	*	This file creates the admin menu for easier recognition
	*	---------------------------------------------------------------------
	*/

	if( !class_exists('gdlr_core_admin_menu') ){
		class gdlr_core_admin_menu{
			
			static $sub_menu = array();
			static $main_menu = array();

			static function register_menu($settings){
				if( !empty($settings['main-menu']) ){
					// main-menu-title, page-title, menu-title, capability, menu-slug, function, icon-url, $position
					self::$main_menu[$settings['menu-slug']] = $settings;
				}else{
					// parent-slug, $page-title, menu-title, capability, menu-slug, function
					self::$sub_menu[$settings['menu-slug']] = $settings;
				}
			}
			
			// init the action for wordpress
			static function init(){
				// action for creating menu
				add_action('admin_menu', 'gdlr_core_admin_menu::register_admin_menu', 9999);
	
				// action for changing the menu active state
				add_filter('submenu_file', 'gdlr_core_admin_menu::submenu_file', 10, 2);
			}
			
			// register the menu based on elements registered to this class
			static function register_admin_menu(){
				
				// wp-admin/menu.php
				global $menu, $submenu;
				
				// register main menu
				if( !empty(self::$main_menu) ){
					foreach( self::$main_menu as $settings ){
						add_menu_page( $settings['page-title'], $settings['menu-title'], $settings['capability'], 
							$settings['menu-slug'], $settings['function'], $settings['icon-url'], $settings['position'] );
						
						// overwrite menu title ( to be different than sub menu )
						if( !empty($settings['main-menu-title']) ){
							$menu[$settings['position']][0] = $settings['main-menu-title'];
						}
					}
				}
				
				// register submenu
				if( !empty(self::$sub_menu) ){
					foreach( self::$sub_menu as $submenu_key => $settings ){

						if( empty($settings['function']) ){
							
							// unset original menu if defined
							$menu_exists = false;
							if( !empty($submenu[$settings['menu-parent-slug']]) ){
								foreach( $submenu[$settings['menu-parent-slug']] as $key => $menu_item ){
									if( $menu_item[2] == $settings['menu-slug'] ){
										$menu_exists = true;
										if( !empty($settings['unset-original']) ){
											unset($submenu[$settings['menu-parent-slug']][$key]);
										}
									}
								}
							}else{
								foreach( $menu as $key => $menu_item ){
									if( $menu_item[2] == $settings['menu-slug'] ){
										$menu_exists = true;
										if( !empty($settings['unset-original']) ){
											unset($menu[$key]);
										}
									}
								}
							}

							// if menu does not exists, continue
							if( !$menu_exists ){ continue; }

							// create first submenu (parent) if hasn't initiate yet	
							if( empty($submenu[$settings['parent-slug']]) ){
								$submenu[$settings['parent-slug']][] = array(
									self::$main_menu[$settings['parent-slug']]['menu-title'],
									self::$main_menu[$settings['parent-slug']]['capability'],
									self::$main_menu[$settings['parent-slug']]['menu-slug'],
									self::$main_menu[$settings['parent-slug']]['page-title'] 
								);
							}
							
							// add the submenu
							$submenu[$settings['parent-slug']][] = array(
								$settings['menu-title'],
								$settings['capability'],
								empty($settings['menu-url'])? $settings['menu-slug']: $settings['menu-url'],
								$settings['page-title']
							);
							
						}else{
							
							// adding new sub menu page
							add_submenu_page( $settings['parent-slug'], $settings['page-title'], $settings['menu-title'], 
								$settings['capability'], $settings['menu-slug'], $settings['function'] );
						}
					}
				}
				
			} // register_admin_menu
		
			// action for changing the menu active state
			static function submenu_file($submenu_file, $p_file){

				global $parent_file, $plugin_page, $pagenow, $self, $_wp_real_parent_file; 

				foreach( self::$sub_menu as $settings ){
					if( empty($settings['function']) && !empty(self::$main_menu[$settings['parent-slug']]) && 
						!empty($settings['menu-url']) && $plugin_page == $settings['menu-slug'] ){

						$pagenow = 'admin.php';

						$_wp_real_parent_file[$parent_file] = $settings['parent-slug'];
						$parent_file = $self = $settings['parent-slug'];
						return $settings['menu-url'];				
					}
				}
				
				return $submenu_file;
				
			} // submenu_file		
		
		} // gdlr_core_admin_menu
	} // class_exists