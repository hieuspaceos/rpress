<?php
	/*	
	*	Goodlayers Item For Page Builder
	*/
	
	if( !class_exists('gdlr_core_pb_wrapper_background') ){
		class gdlr_core_pb_wrapper_background{
			
			// get the element settings
			static function get_settings(){
				return array(	
					'feature' => true,
					'type' => 'background',
					'title' => esc_html__('Wrapper', 'goodlayers-core'),
					'thumbnail' => GDLR_CORE_URL . '/framework/images/page-builder/wrapper.png',
				);
			}
			
			// return the element options
			static function get_options(){
				$options = array(
					'general' => array(
						'title' => esc_html__('General', 'goodlayers-core'),
						'options' => array(
							'privacy' => array(
								'title' => esc_html__('Privacy', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_pb_privacy_options(),
								'description' => esc_html__('Use to omit the content before accepting the consent. You can create privacy settings at the "Theme option > Miscalleneous" area.', 'goodlayers-core')
							),
							'content-layout' => array(
								'title' => esc_html__('Content Layout', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'boxed' => esc_html__('Boxed ( content stays within container area )', 'goodlayers-core'),
									'full' => esc_html__('Full', 'goodlayers-core'),
									'custom' => esc_html__('Custom Width', 'goodlayers-core'),
								)
							),
							'max-width-wrapper' => array(
								'title' => esc_html__('Max Width ( Background )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('Maximum width of this wrapper item.', 'goodlayers-core'),
							),
							'max-width' => array(
								'title' => esc_html__('Max Width ( Container )', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'description' => esc_html__('Maximum width of the container.', 'goodlayers-core'),
								'condition' => array( 'content-layout' => 'custom' )
							),
							'enable-space' => array(
								'title' => esc_html__('Allow Item Padding ( default left / right spaces )', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'content-layout' => 'full' )
							),
							'hide-this-wrapper-in' => array(
								'title' => esc_html__('Hide This Wrapper In', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'desktop' => esc_html__('Desktop', 'goodlayers-core'),
									'desktop-tablet' => esc_html__('Desktop & Tablet', 'goodlayers-core'),
									'tablet' => esc_html__('Tablet', 'goodlayers-core'),
									'tablet-mobile' => esc_html__('Tablet & Mobile', 'goodlayers-core'),
									'mobile' => esc_html__('Mobile', 'goodlayers-core'),
								)
							), 
							'animation' => array(
								'title' => esc_html__('Animation', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => gdlr_core_get_animation_option(),
								'default' => 'none'
							),
							'animation-location' => array(
								'title' => esc_html__('Animation Location', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.8',
								'description' => esc_html__('The location of the object which the animation take places.', 'goodlayers-core') . '<br>' .
									esc_html__('Fill 0 : Animation will start when the item reach the top of the screen', 'goodlayers-core') . '<br>' . 
									esc_html__('Fill 1 : Animation will start when the item is at the bottom of the screen.', 'goodlayers-core')
							),
							'z-index' => array(
								'title' => esc_html__('z-index', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Fill the number to position column in front of another column when it overlaps.', 'goodlayers-core') . 
									' ' . esc_html__('Only applied to front end of the site.', 'goodlayers-core')
							),
						),
					),
					'background' => array(
						'title' => esc_html__('Background', 'goodlayers-core'),
						'options' => array(
							'full-height' => array(
								'title' => esc_html__('Full Height Wrapper', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'description' => esc_html__('Effects will only be shown at the front end of the site', 'goodlayers-core'),
							),
							'decrease-height' => array(
								'title' => esc_html__('Decrease Height Of Full Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'default' => '0px',
								'condition' => array( 'full-height' => 'enable' )
							),
							'centering-content' => array(
								'title' => esc_html__('Center Content At The Middle', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
								'condition' => array( 'full-height' => 'enable' )
							),
							'background-type' => array(
								'title' => esc_html__('Background Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'color' => esc_html__('Color', 'goodlayers-core'),
									'image' => esc_html__('Image', 'goodlayers-core'),
									'video' => esc_html__('Video ( Youtube & Vimeo )', 'goodlayers-core'),
									'html5-video' => esc_html__('Html5 Video', 'goodlayers-core'),
									'pattern' => esc_html__('Pattern', 'goodlayers-core'),
								),
								'default' => 'color'
							),
							'background-color-style' => array(
								'title' => esc_html__('Background Color Type', 'goodlayers'),
								'type' => 'combobox',
								'options' => array(
									'plain' => esc_html__('Plain Color', 'goodlayers-core'),
									'gradient' => esc_html__('Gradient - Vertical', 'goodlayers-core'),
									'gradient-v' => esc_html__('Gradient - Horizontal', 'goodlayers-core'),
								),
								'condition' => array( 'background-type' => array('color', 'image') ),
							),
							'background-color' => array(
								'title' => esc_html__('Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'background-type' => array('color', 'image') ),
								'description' => esc_html__('Can also be use as a base color of an image (with opacity) as well.', 'goodlayers-core')
							),
							'background-color-opacity' => array(
								'title' => esc_html__('Background Color Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => 1,
								'condition' => array( 'background-type' => array('color', 'image'), 'background-color-style' => array('gradient', 'gradient-v') ),
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							), 
							'background-gradient-color' => array(
								'title' => esc_html__('Background Gradient Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'background-type' => array('color', 'image'), 'background-color-style' => array('gradient', 'gradient-v') ),
								'description' => esc_html__('Can also be use as a base color of an image (with opacity) as well.', 'goodlayers-core')
							),
							'background-gradient-color-opacity' => array(
								'title' => esc_html__('Background Gradient Color Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => 1,
								'condition' => array( 'background-type' => array('color', 'image'), 'background-color-style' => array('gradient', 'gradient-v') ),
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'background-image' => array(
								'title' => esc_html__('Background Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'background-type' => 'image' )
							),
							'background-image-top-offset' => array(
								'title' => esc_html__('Background Image Top Offset', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'background-type' => 'image' )
							),
							'background-image-bottom-offset' => array(
								'title' => esc_html__('Background Image Bottom Offset', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array( 'background-type' => 'image' )
							),
							'mobile-background-image' => array(
								'title' => esc_html__('Mobile Background Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'background-type' => 'image' )
							),
							'background-image-style' => array(
								'title' => esc_html__('Background Image Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'cover' => esc_html__('Cover ( full width and height )', 'goodlayers-core'),
									'contain' => esc_html__('Contain', 'goodlayers-core'),
									'no-repeat' => esc_html__('No Repeat', 'goodlayers-core'),
									'repeat' => esc_html__('Repeat X & Y', 'goodlayers-core'),
									'repeat-x' => esc_html__('Repeat X', 'goodlayers-core'),
									'repeat-y' => esc_html__('Repeat Y', 'goodlayers-core'),
								),
								'default' => 'cover',
								'condition' => array( 'background-type' => 'image' )
							), 
							'background-image-position' => array(
								'title' => esc_html__('Background Image Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'center' => esc_html__('Center', 'goodlayers-core'),
									'top-left' => esc_html__('Top Left', 'goodlayers-core'),
									'top-center' => esc_html__('Top Center', 'goodlayers-core'),
									'top-right' => esc_html__('Top Right', 'goodlayers-core'),
									'center-left' => esc_html__('Center Left', 'goodlayers-core'),
									'center-right' => esc_html__('Center Right', 'goodlayers-core'),
									'bottom-left' => esc_html__('Bottom Left', 'goodlayers-core'),
									'bottom-center' => esc_html__('Bottom Center', 'goodlayers-core'),
									'bottom-right' => esc_html__('Bottom Right', 'goodlayers-core'),
									'custom' => esc_html__('Custom', 'goodlayers-core'),
								),
								'default' => 'center',
								'condition' => array( 'background-type' => 'image' )
							),
							'background-image-position-custom' => array(
								'title' => esc_html__('Background Image Postion (Custom)', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y'),
								'data-input-type' => 'pixel',
								'condition' => array( 'background-type' => 'image', 'background-image-position' => 'custom' )
							),
							'background-video-url' => array(
								'title' => esc_html__('Background Video URL', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'background-type' => 'video' ),
							),
							'background-video-url-mp4' => array(
								'title' => esc_html__('Background Video URL (MP4)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'background-type' => 'html5-video' ),
							),
							'background-video-url-webm' => array(
								'title' => esc_html__('Background Video URL (WEBM)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'background-type' => 'html5-video' ),
							),
							'background-video-url-ogg' => array(
								'title' => esc_html__('Background Video URL (ogg)', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'background-type' => 'html5-video' ),
							),
							'background-video-image' => array(
								'title' => esc_html__('Background Image Fallback', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'background-type' => array('video', 'html5-video') ),
								'description' => esc_html__('This background will be showing up when the device you\'re using cannot render the video as background ( eg. mobile device )', 'goodlayers-core'),
							),
							'background-pattern' => array(
								'title' => esc_html__('Background Pattern', 'goodlayers-core'),
								'type' => 'radioimage',
								'options' => 'pattern',
								'default' => 'pattern-1',
								'wrapper-class' => 'gdlr-core-fullsize',
								'condition' => array( 'background-type' => 'pattern' )
							),
							'background-opacity' => array(
								'title' => esc_html__('Background Image Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '',
								'condition' => array( 'background-type' => 'image' ), 
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							), 
							'background-filter' => array(
								'title' => esc_html__('Background Filter', 'goodlayers-core'),
								'type' => 'combobox', 
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'blur' => esc_html__('Blur', 'goodlayers-core')
								),
								'condition' => array('background-type' => 'image')
							),
							'background-blur-size' => array(
								'title' => esc_html__('Background Blur Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array('background-type' => 'image', 'background-filter' => 'blur'),
								'description' => wp_kses(__('See supported browser <a href="https://caniuse.com/#feat=css-filters" target="_blank" >here</a>', 'goodlayers-core'), array('a'=>array('href'=>array(), 'target'=>array())))
							),
							'pattern-opacity' => array(
								'title' => esc_html__('Pattern Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => 1,
								'condition' => array( 'background-type' => 'pattern' ), 
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							), 
							'parallax-speed' => array(
								'title' => esc_html__('Parallax Speed', 'goodlayers-core'),
								'type' => 'text',
								'default' => 0.8,
								'condition' => array( 'background-type' => array('image', 'video', 'html5-video', 'pattern') ), 
								'description' => esc_html__('Fill the number between -1 to 1, ( value 1 equals to background-attachment: fixed )', 'goodlayers-core')
							),
							'overflow' => array(
								'title' => esc_html__('Content Overflow', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'' => esc_html__('Visible', 'goodlayers-core'),
									'hidden' => esc_html__('Hidden', 'goodlayers-core'),
								),
								'default' => 'visible',
								'description' => esc_html__('Set to hidden to cut the content which flowing out of this wrapper. * Effects will only apply to front end of the site.', 'goodlayers-core')
							),
						),
					),
					'container' => array(
						'title' => esc_html__('Container', 'goodlayers-core'),
						'options' => array(
							'enable-container-background' => array(
								'title' => esc_html__('Enable Container Background', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'container-background-color' => array(
								'title' => esc_html__('Container Background Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-container-background' => 'enable' ),
							),
							'container-background-gradient' => array(
								'title' => esc_html__('Container Background Gradient', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'top' => esc_html__('Top ( Fade )', 'goodlayers-core'),
									'bottom' => esc_html__('Bottom ( Fade )', 'goodlayers-core'),
									'gradient' => esc_html__('Vertical', 'goodlayers-core'),
									'gradient-v' => esc_html__('Horizontal', 'goodlayers-core'),
								),
								'condition' => array( 'enable-container-background' => 'enable' )
							),
							'container-background-gradient-min-opacity' => array(
								'title' => esc_html__('Container Background Gradient Min Opacity', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'enable-container-background' => 'enable', 'container-background-gradient' => array('top', 'bottom') )
							),
							'container-background-gradient-color' => array(
								'title' => esc_html__('Container Background Gradient Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-container-background' => 'enable', 'container-background-gradient' => array('gradient', 'gradient-v') ),
							),
							'container-background-image' => array(
								'title' => esc_html__('Container Background Image', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'enable-container-background' => 'enable' )
							),
							'container-background-image-opacity' => array(
								'title' => esc_html__('Container Background Image Opacity', 'goodlayers-core'),
								'type' => 'text',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'enable-container-background' => 'enable' )
							),
							'container-padding' => array(
								'title' => esc_html__('Container Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-container-background' => 'enable' ),
								'default' => array( 'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link' ),
								'description' => esc_html__('Spaces before/after the item inside it. ( including the background area )', 'goodlayers-core')
							),
							'container-margin' => array(
								'title' => esc_html__('Container Margin Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-container-background' => 'enable' ),
								'default' => array( 'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link' ),
								'description' => esc_html__('Spaces before/after this wrapper\'s container. ( outside the background area )', 'goodlayers-core')
							),
							'container-tablet-padding' => array(
								'title' => esc_html__('Container Tablet Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'description' => esc_html__('Breaks at 999px', 'goodlayers-core'),
								'condition' => array( 'enable-container-background' => 'enable' )
							),
							'container-tablet-margin' => array(
								'title' => esc_html__('Container Tablet Margin Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'description' => esc_html__('Breaks at 999px', 'goodlayers-core'),
								'condition' => array( 'enable-container-background' => 'enable' )
							),
							'container-mobile-padding' => array(
								'title' => esc_html__('Container Mobile Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'description' => esc_html__('Breaks at 767px', 'goodlayers-core'),
								'condition' => array( 'enable-container-background' => 'enable' )
							),
							'container-mobile-margin' => array(
								'title' => esc_html__('Container Mobile Margin Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'description' => esc_html__('Breaks at 767px', 'goodlayers-core'),
								'condition' => array( 'enable-container-background' => 'enable' )
							),
							'container-border-radius' => array(
								'title' => esc_html__('Container Border Radius', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('t-left', 't-right', 'b-right', 'b-left'),
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-container-background' => 'enable' )
							),
							'container-shadow-size' => array(
								'title' => esc_html__('Container Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
								'condition' => array( 'enable-container-background' => 'enable' ),
							),
							'container-shadow-color' => array(
								'title' => esc_html__('Container Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array( 'enable-container-background' => 'enable' ),
							),
							'container-shadow-opacity' => array(
								'title' => esc_html__('Container Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core'),
								'condition' => array( 'enable-container-background' => 'enable' ),
							),
							'container-z-index' => array(
								'title' => esc_html__('Container Z-Index', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array( 'enable-container-background' => 'enable' ),
								'description' => esc_html__('Only applied to front end of the site', 'goodlayers-core')
							),
						)
					),
					'effect' => array(
						'title' => esc_html__('Effects', 'goodlayers-core'),
						'options' => array(
							'wrapper-background-gradient' => array(
								'title' => esc_html__('Wrapper Background Top/Bottom Gradient', 'kingster'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'kingster'),
									'both' => esc_html__('Both', 'kingster'),
									'top' => esc_html__('Top', 'kingster'),
									'bottom' => esc_html__('Bottom', 'kingster'),
								),
								'default' => 'none',
							),
							'wrapper-background-gradient-color' => array(
								'title' => esc_html__('Wrapper Background Gradient Color', 'kingster'),
								'type' => 'colorpicker',
								'default' => '#000',
								'condition' => array( 'wrapper-background-gradient' => array('top', 'bottom', 'both') )
							),
							'wrapper-background-gradient-opacity' => array(
								'title' => esc_html__('Wrapper Background Gradient Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '1',
								'condition' => array( 'wrapper-background-gradient' => array('top', 'bottom', 'both') ),
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
							'wrapper-background-top-gradient-size' => array(
								'title' => esc_html__('Single Blog Title Top Gradient Size', 'traveltour'),
								'type' => 'fontslider',
								'data-type' => 'pixel',
								'data-min' => '0',
								'data-max' => '1000',
		 						'default' => '413px',
								'condition' => array( 'wrapper-background-gradient' => array('top', 'both') )
							),
							'wrapper-background-bottom-gradient-size' => array(
								'title' => esc_html__('Single Blog Title Bottom Gradient Size', 'goodlayers-core'),
								'type' => 'fontslider',
								'data-type' => 'pixel',
								'data-min' => '0',
								'data-max' => '1000',
		 						'default' => '413px',
								'condition' => array( 'wrapper-background-gradient' => array('bottom', 'both') )
							),
							'enable-background-image-overlay' => array(
								'title' => esc_html__('Enable Background Image Overlay', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable',
							),
							'background-image-overlay' => array(
								'title' => esc_html__('Background Image Overlay', 'goodlayers-core'),
								'type' => 'upload',
								'condition' => array( 'enable-background-image-overlay' => 'enable' ),
							),
							'background-image-overlay-section' => array(
								'title' => esc_html__('Background Image Overlay Section', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'left' => esc_html__('Left', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core'),
								),
								'condition' => array( 'enable-background-image-overlay' => 'enable' ),
							),
							'background-image-overlay-margin' => array(
								'title' => esc_html__('Background Image Overlay Margin', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link' ),
								'condition' => array( 'enable-background-image-overlay' => 'enable' ),
							), 
							'enable-background-particle' => array(
								'title' => esc_html__('Enable Background Particle', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'enable-marquee' => array(
								'title' => esc_html__('Enable Background Marquee', 'goodlayers-core'),
								'type' => 'checkbox',
								'default' => 'disable'
							),
							'marquee-text' => array(
								'title' => esc_html__('Background Marquee Text', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array('enable-marquee' => 'enable')
							),
							'marquee-position' => array(
								'title' => esc_html__('Background Marquee Position', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'top' => esc_html__('Top', 'goodlayers-core'),
									'bottom' => esc_html__('Bottom', 'goodlayers-core'),
								),
								'condition' => array('enable-marquee' => 'enable')
							),
							'marquee-position-offset' => array(
								'title' => esc_html__('Background Marquee Position Offset', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array('enable-marquee' => 'enable')
							),
							'marquee-speed' => array(
								'title' => esc_html__('Background Marquee Scroll Duration (Millisec)', 'goodlayers-core'),
								'type' => 'text',
								'default' => '10000',
								'condition' => array('enable-marquee' => 'enable')
							),
							'marquee-direction' => array(
								'title' => esc_html__('Background Marquee Scroll Direction', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'left' => esc_html__('Left', 'goodlayers-core'),
									'right' => esc_html__('Right', 'goodlayers-core'),
								),
								'condition' => array('enable-marquee' => 'enable')
							),
							'marquee-font-size' => array(
								'title' => esc_html__('Background Marquee Font Size', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array('enable-marquee' => 'enable')
							),
							'marquee-font-weight' => array(
								'title' => esc_html__('Background Marquee Font Weight', 'goodlayers-core'),
								'type' => 'text',
								'condition' => array('enable-marquee' => 'enable'),
								'description' => esc_html__('Eg. lighter, bold, normal, 300, 400, 600, 700, 800', 'goodlayers-core')
							),
							'marquee-font-letter-spacing' => array(
								'title' => esc_html__('Background Marquee Font Letter Spacing', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
								'condition' => array('enable-marquee' => 'enable')
							),
							'marquee-text-color' => array(
								'title' => esc_html__('Background Marquee Text Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'condition' => array('enable-marquee' => 'enable')
							),
							'marquee-opacity' => array(
								'title' => esc_html__('Background Marquee Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '1',
								'condition' => array('enable-marquee' => 'enable'),
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
						)
					),
					'border' => array(
						'title' => esc_html__('Border', 'goodlayers-core'),
						'options' => array(
							'border-radius' => array(
								'title' => esc_html__('Border Radius', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('t-left', 't-right', 'b-right', 'b-left'),
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
							),
							'border-type' => array(
								'title' => esc_html__('Border Type', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'none' => esc_html__('None', 'goodlayers-core'),
									'outer-border' => esc_html__('Outer Border', 'goodlayers-core'),
									'inner-border' => esc_html__('Inner Border', 'goodlayers-core'),
								),
								'default' => 'none'
							),
							'border-pre-spaces' => array(
								'title' => esc_html__('Spaces Before Inner Border', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'20px', 'right'=>'20px', 'bottom'=>'20px', 'left'=>'20px', 'settings'=>'link' ),
								'condition' => array( 'border-type' => 'inner-border' ),
							), 
							'border-width' => array(
								'title' => esc_html__('Border Width', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'1px', 'right'=>'1px', 'bottom'=>'1px', 'left'=>'1px', 'settings'=>'link' ),
								'condition' => array( 'border-type' => array('outer-border', 'inner-border') ),
							),
							'border-color' => array(
								'title' => esc_html__('Border Color', 'goodlayers-core'),
								'type' => 'colorpicker',
								'default' => '#ffffff',
								'condition' => array( 'border-type' => array('outer-border', 'inner-border') ),
							),
							'border-style' => array(
								'title' => esc_html__('Border Style', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => array(
									'solid' => esc_html__('solid', 'goodlayers-core'),
									'dotted' => esc_html__('dotted', 'goodlayers-core'),
									'dashed' => esc_html__('dashed', 'goodlayers-core'),
									'double' => esc_html__('double', 'goodlayers-core'),
									'groove' => esc_html__('groove', 'goodlayers-core'),
									'ridge' => esc_html__('ridge', 'goodlayers-core'),
									'inset' => esc_html__('inset', 'goodlayers-core'),
									'outset' => esc_html__('outset', 'goodlayers-core'),
								),
								'default' => 'solid',
								'condition' => array( 'border-type' => array('outer-border', 'inner-border') ),
							),
							'background-shadow-size' => array(
								'title' => esc_html__('Background Shadow Size', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'options' => array('x', 'y', 'size'),
								'data-input-type' => 'pixel',
							),
							'background-shadow-color' => array(
								'title' => esc_html__('Background Shadow Color', 'goodlayers-core'),
								'type' => 'colorpicker'
							),
							'background-shadow-opacity' => array(
								'title' => esc_html__('Background Shadow Opacity', 'goodlayers-core'),
								'type' => 'text',
								'default' => '0.2',
								'description' => esc_html__('Fill the number between 0.01 to 1', 'goodlayers-core')
							),
						)
					),
					'spacing' => array(
						'title' => esc_html__('Spacing', 'goodlayers-core'),
						'options' => array(
							'min-height' => array(
								'title' => esc_html__('Min Height', 'goodlayers-core'),
								'type' => 'text',
								'data-input-type' => 'pixel',
							),
							'padding' => array(
								'title' => esc_html__('Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' ),
								'description' => esc_html__('Spaces before/after the item inside it. ( including the background area )', 'goodlayers-core')
							),
							'margin' => array(
								'title' => esc_html__('Margin Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link' ),
								'description' => esc_html__('Spaces before/after this wrapper. ( outside the background area )', 'goodlayers-core')
							),
							'tablet-padding' => array(
								'title' => esc_html__('Tablet Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'description' => esc_html__('Breaks at 999px', 'goodlayers-core')
							),
							'tablet-margin' => array(
								'title' => esc_html__('Tablet Margin Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'description' => esc_html__('Breaks at 999px', 'goodlayers-core')
							),
							'mobile-padding' => array(
								'title' => esc_html__('Mobile Padding Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'description' => esc_html__('Breaks at 767px', 'goodlayers-core')
							),
							'mobile-margin' => array(
								'title' => esc_html__('Mobile Margin Spaces', 'goodlayers-core'),
								'type' => 'custom',
								'item-type' => 'padding',
								'data-input-type' => 'pixel',
								'default' => array( 'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' ),
								'description' => esc_html__('Breaks at 767px', 'goodlayers-core')
							),
						),
					),
					'skin' => array(
						'title' => esc_html__('Custom Skin', 'goodlayers-core'),
						'options' => array(
							'skin' => array(
								'title' => esc_html__('Skin', 'goodlayers-core'),
								'type' => 'combobox',
								'options' => 'skin'
							),						
						)
					)

				);
				
				$extra_options = apply_filters('gdlr_core_pb_wrapper_background_extra', array());
				if( in_array('float-social-id', $extra_options) ){
					$options['general']['options']['float-social-id'] = array(
						'title' => esc_html__('Float Social ID', 'goodlayers-core'),
						'type' => 'text',
					);
				}

				return $options;
			}			
			
			// get element template for page builder
			static function get_template( $options = array(), $callback = '' ){

				$wrapper_style_atts = array(
					'margin' => (empty($options['value']['margin']) || $options['value']['margin'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? array(): $options['value']['margin'],
					'padding' => (empty($options['value']['padding']) || $options['value']['padding'] == array( 
							'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' 
						))? '': $options['value']['padding'],
					'min-height' => (empty($options['value']['min-height'])? '': $options['value']['min-height'])
				);
				if( !empty($options['value']['max-width-wrapper']) ){
					$wrapper_style_atts['max-width'] = $options['value']['max-width-wrapper'];
					$wrapper_style_atts['margin']['left'] = 'auto';
					$wrapper_style_atts['margin']['right'] = 'auto';
				}
				$content_style_atts = array();
				
				// for background
				$wrapper_background = '';
				if( !empty($options['value']['background-type']) ){
					
					// color background
					if( $options['value']['background-type'] == 'color' ){
						$wrapper_style_atts = $wrapper_style_atts + array(
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
									'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
								))? '': $options['value']['border-radius'],
							'background-color' => empty($options['value']['background-color'])? '': $options['value']['background-color'],
							'background-shadow-size' => empty($options['value']['background-shadow-size'])? '': $options['value']['background-shadow-size'],
							'background-shadow-color' => empty($options['value']['background-shadow-color'])? '': $options['value']['background-shadow-color'],
							'background-shadow-opacity' => empty($options['value']['background-shadow-opacity'])? '': $options['value']['background-shadow-opacity'],
						);
						if( !empty($options['value']['background-color-style']) && in_array($options['value']['background-color-style'], array('gradient', 'gradient-v')) ){
							$wrapper_style_atts[$options['value']['background-color-style']] = array(
								$options['value']['background-color'],
								$options['value']['background-gradient-color']
							);
						} 

					// image background
					}else if( $options['value']['background-type'] == 'image' ){
						$bgi_atts = array(
							'opacity' => empty($options['value']['background-opacity'])? '': $options['value']['background-opacity'],
							'background-image' => empty($options['value']['background-image'])? '': $options['value']['background-image'],
						);
						if( !empty($options['value']['background-image-style']) ){
							if( $options['value']['background-image-style'] == 'cover' ){
								$bgi_atts['background-size'] = 'cover';
								$bgi_atts['background-repeat'] = 'no-repeat';
							}else if( $options['value']['background-image-style'] == 'contain' ){
								$bgi_atts['background-size'] = 'contain';
							}else{
								$bgi_atts['background-repeat'] = $options['value']['background-image-style'];
							}
							if( !empty($options['value']['background-image-position']) ){
								if( $options['value']['background-image-position'] == 'custom' ){
									if( !empty($options['value']['background-image-position-custom']) ){
										$bgi_atts['background-position'] = $options['value']['background-image-position-custom']['x'] . ' ' . $options['value']['background-image-position-custom']['y'];
									}
								}else{
									$bgi_atts['background-position'] = str_replace('-', ' ', $options['value']['background-image-position']);
								}
							}
						}
						if( !empty($options['value']['parallax-speed']) && $options['value']['parallax-speed'] == 1 ){
							$bgi_atts['background-attachment'] = 'fixed';
 						}
 						if( !empty($options['value']['background-filter']) && $options['value']['background-filter'] == 'blur' ){
 							if( !empty($options['value']['background-blur-size']) ){
 								$bgi_atts['blur'] = $options['value']['background-blur-size'];
 							}
 						}

						$wrapper_style_atts['background-shadow-size'] = empty($options['value']['background-shadow-size'])? '': $options['value']['background-shadow-size'];
						$wrapper_style_atts['background-shadow-color'] = empty($options['value']['background-shadow-color'])? '': $options['value']['background-shadow-color'];
						$wrapper_style_atts['background-shadow-opacity'] = empty($options['value']['background-shadow-opacity'])? '': $options['value']['background-shadow-opacity'];
						$wrapper_style_atts['border-radius'] = (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
							'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
						))? '': $options['value']['border-radius'];
						
						$wrapper_background_atts = array(
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
						);
						$wrapper_background_atts['background-color'] = empty($options['value']['background-color'])? '': $options['value']['background-color'];
						if( !empty($options['value']['background-color-style']) && in_array($options['value']['background-color-style'], array('gradient', 'gradient-v')) ){
							if( !empty($options['value']['background-gradient-color']) ){
								$wrapper_background_atts[$options['value']['background-color-style']] = array(
									array(
										$options['value']['background-color'],
										empty($options['value']['background-color-opacity'])? 0: $options['value']['background-color-opacity']
									),
									array(
										$options['value']['background-gradient-color'],
										empty($options['value']['background-gradient-color-opacity'])? 0: $options['value']['background-gradient-color-opacity']
									)
								);
							}
						} 

						$wrapper_background = '';
						if( !empty($options['value']['background-image-bottom-offset']) || !empty($options['value']['background-image-top-offset']) ){
							$wrapper_background .= '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style($wrapper_background_atts) . ' ></div>';
							unset($wrapper_background_atts['background-color']);
							unset($wrapper_background_atts['gradient']);
							unset($wrapper_background_atts['gradient-v']);
							if( !empty($options['value']['background-image-bottom-offset']) ){
								$wrapper_background_atts['bottom'] = $options['value']['background-image-bottom-offset'];
							}
							if( !empty($options['value']['background-image-top-offset']) ){
								$wrapper_background_atts['top'] = $options['value']['background-image-top-offset'];
							}
						}

						$wrapper_background .= '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style($wrapper_background_atts) . ' >';
						if( empty($bgi_atts['background-attachment']) ){
							$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style($bgi_atts);
							$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
							$wrapper_background .= ' ></div>';
						}else{
							$wrapper_background .= '<div class="gdlr-core-pbf-background" ' . gdlr_core_esc_style($bgi_atts) . ' ></div>';
						}
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					
					// pattern background
					}else if( $options['value']['background-type'] == 'pattern' ){
						$bgi_atts = array(
							'background-image' => GDLR_CORE_URL . '/include/images/pattern/' . (empty($options['value']['background-pattern'])? 'pattern-1': $options['value']['background-pattern']) . '.png',
							'background-repeat' => 'repeat',
							'background-position' => 'center',
							'opacity' => empty($options['value']['pattern-opacity'])? '1': $options['value']['pattern-opacity'],
						);
						if( !empty($options['value']['parallax-speed']) && $options['value']['parallax-speed'] == 1 ){
							$bgi_atts['background-attachment'] = 'fixed';
 						}
						
						$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style(array(
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
							'background-shadow-size' => empty($options['value']['background-shadow-size'])? '': $options['value']['background-shadow-size'],
							'background-shadow-color' => empty($options['value']['background-shadow-color'])? '': $options['value']['background-shadow-color'],
							'background-shadow-opacity' => empty($options['value']['background-shadow-opacity'])? '': $options['value']['background-shadow-opacity'],
						)) . ' >';
						if( empty($bgi_atts['background-attachment']) ){
							$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style($bgi_atts);
							$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
							$wrapper_background .= ' ></div>';
						}else{
							$wrapper_background .= '<div class="gdlr-core-pbf-background" ' . gdlr_core_esc_style($bgi_atts) . ' ></div>';
						}
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					
					// video background
					}else{
						$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style(array(
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
							'background-shadow-size' => empty($options['value']['background-shadow-size'])? '': $options['value']['background-shadow-size'],
							'background-shadow-color' => empty($options['value']['background-shadow-color'])? '': $options['value']['background-shadow-color'],
							'background-shadow-opacity' => empty($options['value']['background-shadow-opacity'])? '': $options['value']['background-shadow-opacity'],
						)) . ' >';
						$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style(array(
							'background-position' => 'center',
							'background-size' => 'cover',
						));
						$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
						$wrapper_background .= ' data-video-fallback="' . (empty($options['value']['background-video-image'])? '': gdlr_core_get_image_url($options['value']['background-video-image'])) . '" ';
						$wrapper_background .= ' >';
						$wrapper_background .= '<div class="gdlr-core-pbf-background-video" data-background-type="video" >';
						
						if( $options['value']['background-type'] == 'video' ){
							if( !empty($options['value']['background-video-url']) ){
								$wrapper_background .= gdlr_core_get_video(
									$options['value']['background-video-url'], 
									array('width' => '100%', 'height' => '100%'), 
									array('background' => 1)
								);
							}
						}else if( $options['value']['background-type'] == 'html5-video' ){
							$wrapper_background .= '<video autoplay loop muted >';
							if( $options['value']['background-video-url-mp4'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-mp4']) . '" type="video/mp4">';
							}
							if( $options['value']['background-video-url-webm'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-webm']) . '" type="video/webm">';
							}
							if( $options['value']['background-video-url-ogg'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-ogg']) . '" type="video/ogg">';
							}
							$wrapper_background .= '</video>';
						}
						$wrapper_background .= '</div>';
						$wrapper_background .= '</div>'; // gdlr-cpre-pbc-background
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					}
				}

				// border
				if( !empty($options['value']['border-type']) ){
					if( $options['value']['border-type'] == 'outer-border' ){
						$wrapper_style_atts['border-width'] = empty($options['value']['border-width'])? '': $options['value']['border-width'];
						$wrapper_style_atts['border-color'] = empty($options['value']['border-color'])? '': $options['value']['border-color'];
						$wrapper_style_atts['border-style'] = empty($options['value']['border-style'])? '': $options['value']['border-style'];
					}else if( $options['value']['border-type'] == 'inner-border' ){
						$wrapper_background .= '<div class="gdlr-core-pbf-background-frame" ' . gdlr_core_esc_style(array(
							'margin' => (empty($options['value']['border-pre-spaces']))? '0px': $options['value']['border-pre-spaces'],
							'border-width' => empty($options['value']['border-width'])? '': $options['value']['border-width'],
							'border-style' => empty($options['value']['border-style'])? '': $options['value']['border-style'],
							'border-color' => empty($options['value']['border-color'])? '': $options['value']['border-color']
						)) . ' ></div>';
					}
				}	

				// marquee effect
				$marquee_background = '';
				if( !empty($options['value']['enable-marquee']) && $options['value']['enable-marquee'] == 'enable' &&
					!empty($options['value']['marquee-text']) ){
					$marquee_atts = array(
						'color' => empty($options['value']['marquee-text-color'])? '': $options['value']['marquee-text-color'],
						'opacity' => empty($options['value']['marquee-opacity'])? '': $options['value']['marquee-opacity'],
						'font-size' => empty($options['value']['marquee-font-size'])? '': $options['value']['marquee-font-size'],
						'font-weight' => empty($options['value']['marquee-font-weight'])? '': $options['value']['marquee-font-weight'],
						'letter-spacing' => empty($options['value']['marquee-font-letter-spacing'])? '': $options['value']['marquee-font-letter-spacing'],
					);
					if( !empty($options['value']['marquee-position']) && !empty($options['value']['marquee-position-offset']) ){
						if( $options['value']['marquee-position'] == 'top' ){
							$marquee_atts['top'] = $options['value']['marquee-position-offset'];
							$marquee_atts['bottom'] = 'auto';
						}else{
							$marquee_atts['bottom'] = $options['value']['marquee-position-offset'];
							$marquee_atts['top'] = 'auto';
						}
						
					}

					$marquee_background  = '<div class="gdlr-core-pbf-wrapper-marquee gdlr-core-marquee gdlr-core-title-font gdlr-core-js" ';
					$marquee_background .= empty($options['value']['marquee-speed'])? '': 'data-speed="' . esc_attr($options['value']['marquee-speed']) . '" ';
					$marquee_background .= empty($options['value']['marquee-direction'])? '': 'data-direction="' . esc_attr($options['value']['marquee-direction']) . '" ';
					$marquee_background .= gdlr_core_esc_style($marquee_atts);
					$marquee_background .= ' >' . gdlr_core_text_filter($options['value']['marquee-text']) . '</div>';
				} 

				$wrapper  = '<div class="gdlr-core-page-builder-wrapper gdlr-core-page-builder-background-wrapper" data-template="wrapper" data-type="background" ';
				$wrapper .= (empty($options['value'])? '': 'data-value="' . esc_attr(json_encode($options['value'])) . '" ');
				$wrapper .= (empty($options['sync-template'])? '': 'data-sync-template="' . esc_attr($options['sync-template']) . '" ');
				$wrapper .= (empty($options['value']['skin'])? '': 'data-skin="' . esc_attr($options['value']['skin']) . '" ');
				$wrapper .= '>';
				
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-content" >';
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-head">';
				$wrapper .= '<span class="gdlr-core-page-builder-wrapper-head-title" >' . esc_html__('Wrapper', 'goodlayers-core') . '</span>';
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-settings">';
				$wrapper .= '<i class="fa fa-edit gdlr-core-page-builder-wrapper-edit" ></i>';
				$wrapper .= '<i class="fa fa-copy gdlr-core-page-builder-wrapper-copy" ></i>';
				$wrapper .= '<i class="fa fa-download gdlr-core-page-builder-wrapper-save" ></i>';
				$wrapper .= '<i class="fa fa-refresh gdlr-core-page-builder-wrapper-sync" ></i>';
				$wrapper .= '<i class="fa fa-remove gdlr-core-page-builder-wrapper-remove" ></i>';
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-settings
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-head
				
				$wrapper_display_class  = empty($options['value']['class'])? '': $options['value']['class'];

				$container_class = '';
				$container_style_atts = array();
				if( empty($options['value']['content-layout']) || $options['value']['content-layout'] == 'boxed' ){
					$container_class .= 'gdlr-core-container';
				}else if( $options['value']['content-layout'] == 'custom' ){
					$container_class .= 'gdlr-core-container-custom';
					if( !empty($options['value']['max-width']) ){
						$container_style_atts['max-width'] = $options['value']['max-width'];
					}
				}else if( $options['value']['content-layout'] == 'full' ){
					if( empty($options['value']['enable-space']) || $options['value']['enable-space'] == 'disable' ){
						$container_class .= 'gdlr-core-pbf-wrapper-full-no-space';
					}else{
						$container_class .= 'gdlr-core-pbf-wrapper-full';
					}
				}

				$container_inner_class = '';
				$container_inner_atts = array();
				if( !empty($options['value']['enable-container-background']) && $options['value']['enable-container-background'] == 'enable' ){
					$container_inner_class = ' gdlr-core-pbf-wrapper-container-inner';
					$container_inner_atts = array(
						'margin' => (empty($options['value']['container-margin']) || $options['value']['container-margin'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? array(): $options['value']['container-margin'],
						'padding' => (empty($options['value']['container-padding']) || $options['value']['container-padding'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? array(): $options['value']['container-padding'],
						'background-color' => empty($options['value']['container-background-color'])? '': $options['value']['container-background-color'],
						'border-radius' => empty($options['value']['container-border-radius'])? '': $options['value']['container-border-radius'],
						'background-shadow-size' => empty($options['value']['container-shadow-size'])? '': $options['value']['container-shadow-size'],
						'background-shadow-color' => empty($options['value']['container-shadow-color'])? '': $options['value']['container-shadow-color'],
						'background-shadow-opacity' => empty($options['value']['container-shadow-opacity'])? '': $options['value']['container-shadow-opacity'],
						'z-index' => empty($options['value']['container-z-index'])? '': $options['value']['container-z-index'],
					);

					if( !empty($options['value']['container-background-color']) &&
						!empty($options['value']['container-background-gradient']) && $options['value']['container-background-gradient'] != 'none' ){

						if( in_array($options['value']['container-background-gradient'], array('top', 'bottom')) ){
							$min_gradient = empty($options['value']['container-background-gradient-min-opacity'])? 0: $options['value']['container-background-gradient-min-opacity'];

							$container_inner_atts['gradient'] = array(
								array(
									$options['value']['container-background-color'],
									($options['value']['container-background-gradient'] == 'top')? $min_gradient: 1 
								),
								array(
									$options['value']['container-background-color'],
									($options['value']['container-background-gradient'] == 'bottom')? $min_gradient: 1 
								)
							);
						}else if( in_array($options['value']['container-background-gradient'], array('gradient', 'gradient-v')) && !empty($options['value']['container-background-gradient-color']) ){
							$container_inner_atts[$options['value']['container-background-gradient']] = array(
								$options['value']['container-background-color'],
								$options['value']['container-background-gradient-color']
							);
						}
						
					}

					if( (!empty($options['value']['container-margin']['left']) && $options['value']['container-margin']['left'] != '0px') ||
						(!empty($options['value']['container-margin']['right']) && $options['value']['container-margin']['right'] != '0px') ){

						$container_inner_margin = intval($options['value']['container-margin']['left']) + intval($options['value']['container-margin']['right']);
						$container_inner_atts['width'] = 'calc(100% - ' . $container_inner_margin . 'px)';
					}
				}

				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-content-margin ' . esc_attr($wrapper_display_class) . '" ' . gdlr_core_esc_style($wrapper_style_atts) . ' >';
				$wrapper .= $wrapper_background . $marquee_background;

				// for background gradient
				if( !empty($options['value']['wrapper-background-gradient']) && in_array($options['value']['wrapper-background-gradient'], array('both', 'top', 'bottom')) ){
					if( in_array($options['value']['wrapper-background-gradient'], array('top', 'both')) ){
						$wrapper .= '<div class="gdlr-core-page-builder-wrapper-top-gradient" ' . gdlr_core_esc_style(array(
							'height' => empty($options['value']['wrapper-background-top-gradient-size'])? 0: $options['value']['wrapper-background-top-gradient-size'],
							'opacity' => empty($options['value']['wrapper-background-gradient-opacity'])? '': $options['value']['wrapper-background-gradient-opacity'],
							'gradient' => array(
								array(
									empty($options['value']['wrapper-background-gradient-color'])? '#000': $options['value']['wrapper-background-gradient-color'], 
									100
								),
								array(
									empty($options['value']['wrapper-background-gradient-color'])? '#000': $options['value']['wrapper-background-gradient-color'],
									0
								)
							),
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
						)) . ' ></div>';
					}
					if( in_array($options['value']['wrapper-background-gradient'], array('bottom', 'both')) ){
						$wrapper .= '<div class="gdlr-core-page-builder-wrapper-bottom-gradient" ' . gdlr_core_esc_style(array(
							'height' => empty($options['value']['wrapper-background-bottom-gradient-size'])? 0: $options['value']['wrapper-background-bottom-gradient-size'],
							'opacity' => empty($options['value']['wrapper-background-gradient-opacity'])? '': $options['value']['wrapper-background-gradient-opacity'],
							'gradient' => array(
								array(
									empty($options['value']['wrapper-background-gradient-color'])? '#000': $options['value']['wrapper-background-gradient-color'], 
									0
								),
								array(
									empty($options['value']['wrapper-background-gradient-color'])? '#000': $options['value']['wrapper-background-gradient-color'],
									100
								)
							),
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
						)) . ' ></div>';
					}
				}	

				// overlay background
				if( !empty($options['value']['enable-background-image-overlay']) && $options['value']['enable-background-image-overlay'] == 'enable' ){
					$overlay_background  = '<div class="gdlr-core-wrapper-bg-overlay ';
					$overlay_background .= 'gdlr-core-pos-' . (empty($options['value']['background-image-overlay-section'])? 'left': $options['value']['background-image-overlay-section']) . ' ';
					$overlay_background .= '" ' . gdlr_core_esc_style(array(
						'background-image' => empty($options['value']['background-image-overlay'])? '': $options['value']['background-image-overlay'],
						'margin' => (empty($options['value']['background-image-overlay-margin']) || $options['value']['background-image-overlay-margin'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? array(): $options['value']['background-image-overlay-margin'],
					)) . ' ></div>';
					
					$wrapper .= $overlay_background;
				}

				if( !empty($options['value']['enable-background-particle']) && $options['value']['enable-background-particle'] == 'enable' ){
					$wrapper .= '<div class="gdlr-core-js gdlr-core-particle-bg" ></div>';
				}
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-content-wrap gdlr-core-pbf-wrapper-content gdlr-core-js" ' . gdlr_core_esc_style($content_style_atts) . ' ';
				$wrapper .= gdlr_core_get_animation_atts(array(
					'animation' => (empty($options['value']['animation'])? '': $options['value']['animation']),
					'location' => (empty($options['value']['animation-location'])? '': $options['value']['animation-location'])
				)) . ' >';
				$wrapper .= '<div class="' . esc_attr($container_class) . ' clearfix" ' . gdlr_core_esc_style($container_style_atts) . '>';
				$wrapper .= '<div class="gdlr-core-page-builder-wrapper-container gdlr-core-wrapper-sortable clearfix ' . $container_inner_class . '" ' . gdlr_core_esc_style($container_inner_atts) . '>';
				if( !empty($options['value']['container-background-image']) ){
					$wrapper .= '<div class="gdlr-core-page-builder-wrapper-container-bg-image" ' . gdlr_core_esc_style(array(
						'background-image' => $options['value']['container-background-image'],
						'opacity' => empty($options['value']['container-background-image-opacity'])? '': $options['value']['container-background-image-opacity'],
						'border-radius' => empty($options['value']['container-border-radius'])? '': $options['value']['container-border-radius'],
					)) . ' ></div>';
				}
				if( !empty($options['items']) ){
					$wrapper .= gdlr_core_escape_content(call_user_func($callback, $options['items']));
				}
				$wrapper .= '</div>'; // container_class
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-container
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-content-wrap
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-content-margin
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper-content

				// script for background wrapper
				$id = mt_rand(0, 9999);
				
				ob_start();
?><script id="gdlr-core-preview-background-<?php echo esc_attr($id); ?>" >
jQuery(document).ready(function(){
	var background_wrap = jQuery('#gdlr-core-preview-background-<?php echo esc_attr($id); ?>').parent();
	background_wrap.gdlr_core_parallax_background().gdlr_core_particle_background().gdlr_core_marquee().gdlr_core_fluid_video().gdlr_core_ux();
	jQuery(window).trigger('scroll');
});
</script><?php	
				$wrapper .= ob_get_contents();
				ob_end_clean();				
				
				$wrapper .= '</div>'; // gdlr-core-page-builder-wrapper
				
				return $wrapper;
				
			}
			
			// get element content for front end page builder
			static function get_content( $options = array(), $callback = '' ){
				
				// tablet margin / padding
				$mobile_style = '';
				if( !empty($options['value']['tablet-padding']) && $options['value']['tablet-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper {';
					$mobile_style .= gdlr_core_esc_style(array('padding'=>$options['value']['tablet-padding']), false, true);
					$mobile_style .= '} ';
				}
				if( !empty($options['value']['tablet-margin']) && $options['value']['tablet-margin'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper {';
					$mobile_style .= gdlr_core_esc_style(array('margin'=>$options['value']['tablet-margin']), false, true);
					$mobile_style .= '} ';
				}
				if( !empty($options['value']['container-tablet-padding']) && $options['value']['container-tablet-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper .gdlr-core-pbf-wrapper-container-inner{';
					$mobile_style .= gdlr_core_esc_style(array('padding'=>$options['value']['container-tablet-padding']), false, true);
					$mobile_style .= '} ';
				}
				if( !empty($options['value']['container-tablet-margin']) && $options['value']['container-tablet-margin'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper .gdlr-core-pbf-wrapper-container-inner{';
					$mobile_style .= gdlr_core_esc_style(array('margin'=>$options['value']['container-tablet-margin']), false, true);
					$mobile_style .= '} ';
				}
				if( $mobile_style ){
					if( empty($options['value']['id']) ){
						global $gdlr_core_wrapper_id;
						$gdlr_core_wrapper_id = empty($gdlr_core_wrapper_id)? 1: $gdlr_core_wrapper_id + 1;
						$options['value']['id'] = 'gdlr-core-wrapper-' . $gdlr_core_wrapper_id;
					}

					$mobile_style = str_replace('gdlr-core-temp-id', $options['value']['id'], $mobile_style);
					$mobile_style = '@media only screen and (max-width: 999px){' . $mobile_style . '}';
					gdlr_core_add_inline_style($mobile_style);
				}

				// mobile margin / padding
				$mobile_style = '';
				if( !empty($options['value']['mobile-padding']) && $options['value']['mobile-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper {';
					$mobile_style .= gdlr_core_esc_style(array('padding'=>$options['value']['mobile-padding']), false, true);
					$mobile_style .= '} ';
				}
				if( !empty($options['value']['mobile-margin']) && $options['value']['mobile-margin'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper {';
					$mobile_style .= gdlr_core_esc_style(array('margin'=>$options['value']['mobile-margin']), false, true);
					$mobile_style .= '} ';
				}
				if( !empty($options['value']['container-mobile-padding']) && $options['value']['container-mobile-padding'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper .gdlr-core-pbf-wrapper-container-inner{';
					$mobile_style .= gdlr_core_esc_style(array('padding'=>$options['value']['container-mobile-padding']), false, true);
					$mobile_style .= '} ';
				}
				if( !empty($options['value']['container-mobile-margin']) && $options['value']['container-mobile-margin'] != array('top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link') ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper .gdlr-core-pbf-wrapper-container-inner{';
					$mobile_style .= gdlr_core_esc_style(array('margin'=>$options['value']['container-mobile-margin']), false, true);
					$mobile_style .= '} ';
				}
				if( !empty($options['value']['background-type']) && $options['value']['background-type'] == 'image' && !empty($options['value']['mobile-background-image']) ){
					$mobile_style .= '#gdlr-core-temp-id.gdlr-core-pbf-wrapper > .gdlr-core-pbf-background-wrap > .gdlr-core-pbf-background{';
					$mobile_style .= gdlr_core_esc_style(array('background-image'=>$options['value']['mobile-background-image']), false, true);
					$mobile_style .= '} ';
				}
				if( $mobile_style ){
					if( empty($options['value']['id']) ){
						global $gdlr_core_wrapper_id;
						$gdlr_core_wrapper_id = empty($gdlr_core_wrapper_id)? 1: $gdlr_core_wrapper_id + 1;
						$options['value']['id'] = 'gdlr-core-wrapper-' . $gdlr_core_wrapper_id;
					}

					$mobile_style = str_replace('gdlr-core-temp-id', $options['value']['id'], $mobile_style);
					$mobile_style = '@media only screen and (max-width: 767px){' . $mobile_style . '}';
					gdlr_core_add_inline_style($mobile_style);
				}

				// print_item
				$wrapper_style_atts = array(
					'margin' => (empty($options['value']['margin']) || $options['value']['margin'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? array(): $options['value']['margin'],
					'padding' => (empty($options['value']['padding']) || $options['value']['padding'] == array( 
						'top'=>'60px', 'right'=>'0px', 'bottom'=>'30px', 'left'=>'0px', 'settings'=>'unlink' 
					))? '': $options['value']['padding'],
					'min-height' => (empty($options['value']['min-height'])? '': $options['value']['min-height']),
					'overflow' => (empty($options['value']['overflow']) || $options['value']['overflow'] == 'visible')? '': $options['value']['overflow'],
					'z-index' => empty($options['value']['z-index'])? '': $options['value']['z-index']
				);
				if( !empty($options['value']['max-width-wrapper']) ){
					$wrapper_style_atts['max-width'] = $options['value']['max-width-wrapper'];
					$wrapper_style_atts['margin']['left'] = 'auto';
					$wrapper_style_atts['margin']['right'] = 'auto';
				}
				$content_style_atts = array();

				// for background
				$wrapper_background = '';
				if( !empty($options['value']['background-type']) ){
					
					// color background
					if( $options['value']['background-type'] == 'color' ){
						$wrapper_background_atts = array(
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
						);
						if( !empty($options['value']['background-shadow-size']) || !empty($options['value']['background-shadow-color']) || !empty($options['value']['background-shadow-opacity']) ){
							$wrapper_background_atts = $wrapper_background_atts + array(
								'background-shadow-size' => empty($options['value']['background-shadow-size'])? '': $options['value']['background-shadow-size'],
								'background-shadow-color' => empty($options['value']['background-shadow-color'])? '': $options['value']['background-shadow-color'],
								'background-shadow-opacity' => empty($options['value']['background-shadow-opacity'])? '': $options['value']['background-shadow-opacity']
							);
						}
						if( !empty($options['value']['background-color']) ){
							$wrapper_background_atts = $wrapper_background_atts + array(
								'background-color' => $options['value']['background-color'],
							);
							if( !empty($options['value']['background-color-style']) && in_array($options['value']['background-color-style'], array('gradient', 'gradient-v')) ){
								if( !empty($options['value']['background-gradient-color']) ){
									$wrapper_background_atts[$options['value']['background-color-style']] = array(
										array(
											$options['value']['background-color'],
											empty($options['value']['background-color-opacity'])? 0: $options['value']['background-color-opacity']
										),
										array(
											$options['value']['background-gradient-color'],
											empty($options['value']['background-gradient-color-opacity'])? 0: $options['value']['background-gradient-color-opacity']
										)
									);
								}
							}
						}

						if( !empty($wrapper_background_atts) ){
							$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style($wrapper_background_atts) . ' ></div>';
						}
					
					// image background
					}else if( $options['value']['background-type'] == 'image' ){
						$bgi_atts = array(
							'opacity' => empty($options['value']['background-opacity'])? '': $options['value']['background-opacity'],
							'background-image' => empty($options['value']['background-image'])? '': $options['value']['background-image'],
						);
						if( !empty($options['value']['background-image-style']) ){
							if( $options['value']['background-image-style'] == 'cover' ){
								$bgi_atts['background-size'] = 'cover';
								$bgi_atts['background-repeat'] = 'no-repeat';
							}else if( $options['value']['background-image-style'] == 'contain' ){
								$bgi_atts['background-size'] = 'contain';
							}else{
								$bgi_atts['background-repeat'] = $options['value']['background-image-style'];
							}
							if( !empty($options['value']['background-image-position']) ){
								if( $options['value']['background-image-position'] == 'custom' ){
									if( !empty($options['value']['background-image-position-custom']) ){
										$bgi_atts['background-position'] = $options['value']['background-image-position-custom']['x'] . ' ' . $options['value']['background-image-position-custom']['y'];
									}
								}else{
									$bgi_atts['background-position'] = str_replace('-', ' ', $options['value']['background-image-position']);
								}
							}
						}
						if( !empty($options['value']['parallax-speed']) && $options['value']['parallax-speed'] == 1 ){
							$bgi_atts['background-attachment'] = 'fixed';
 						}
 						if( !empty($options['value']['background-filter']) && $options['value']['background-filter'] == 'blur' ){
 							if( !empty($options['value']['background-blur-size']) ){
 								$bgi_atts['blur'] = $options['value']['background-blur-size'];
 							}
 						}
						
						$wrapper_style_atts['background-shadow-size'] = empty($options['value']['background-shadow-size'])? '': $options['value']['background-shadow-size'];
						$wrapper_style_atts['background-shadow-color'] = empty($options['value']['background-shadow-color'])? '': $options['value']['background-shadow-color'];
						$wrapper_style_atts['background-shadow-opacity'] = empty($options['value']['background-shadow-opacity'])? '': $options['value']['background-shadow-opacity'];
						$wrapper_style_atts['border-radius'] = (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
							'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
						))? '': $options['value']['border-radius'];
						
						$wrapper_background_atts = array(
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
						);
						$wrapper_background_atts['background-color'] = $options['value']['background-color'];
						if( !empty($options['value']['background-color-style']) && in_array($options['value']['background-color-style'], array('gradient', 'gradient-v')) ){
							if( !empty($options['value']['background-gradient-color']) ){
								$wrapper_background_atts[$options['value']['background-color-style']] = array(
									array(
										$options['value']['background-color'],
										empty($options['value']['background-color-opacity'])? 0: $options['value']['background-color-opacity']
									),
									array(
										$options['value']['background-gradient-color'],
										empty($options['value']['background-gradient-color-opacity'])? 0: $options['value']['background-gradient-color-opacity']
									)
								);
							}
						}

						$wrapper_background = '';
						if( !empty($options['value']['background-image-bottom-offset']) || !empty($options['value']['background-image-top-offset']) ){
							$wrapper_background .= '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style($wrapper_background_atts) . ' ></div>';
							unset($wrapper_background_atts['background-color']);
							unset($wrapper_background_atts['gradient']);
							unset($wrapper_background_atts['gradient-v']);
							if( !empty($options['value']['background-image-bottom-offset']) ){
								$wrapper_background_atts['bottom'] = $options['value']['background-image-bottom-offset'];
							}
							if( !empty($options['value']['background-image-top-offset']) ){
								$wrapper_background_atts['top'] = $options['value']['background-image-top-offset'];
							}
						}
						$wrapper_background .= '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style($wrapper_background_atts) . ' >';
						if( empty($bgi_atts['background-attachment']) ){
							$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style($bgi_atts);
							$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
							$wrapper_background .= ' ></div>';
						}else{
							$wrapper_background .= '<div class="gdlr-core-pbf-background" ' . gdlr_core_esc_style($bgi_atts) . ' ></div>';
						}
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					
					// pattern background
					}else if( $options['value']['background-type'] == 'pattern' ){
						$bgi_atts = array(
							'background-image' => GDLR_CORE_URL . '/include/images/pattern/' . (empty($options['value']['background-pattern'])? 'pattern-1': $options['value']['background-pattern']) . '.png',
							'background-repeat' => 'repeat',
							'background-position' => 'center',
							'opacity' => empty($options['value']['pattern-opacity'])? '1': $options['value']['pattern-opacity'],
						);
						if( !empty($options['value']['parallax-speed']) && $options['value']['parallax-speed'] == 1 ){
							$bgi_atts['background-attachment'] = 'fixed';
 						}
						
						$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style(array(
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
							'background-shadow-size' => empty($options['value']['background-shadow-size'])? '': $options['value']['background-shadow-size'],
							'background-shadow-color' => empty($options['value']['background-shadow-color'])? '': $options['value']['background-shadow-color'],
							'background-shadow-opacity' => empty($options['value']['background-shadow-opacity'])? '': $options['value']['background-shadow-opacity'],
						)) . ' >';
						if( empty($bgi_atts['background-attachment']) ){
							$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style($bgi_atts);
							$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
							$wrapper_background .= ' ></div>';
						}else{
							$wrapper_background .= '<div class="gdlr-core-pbf-background" ' . gdlr_core_esc_style($bgi_atts) . '></div>';
						}
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					
					// video background
					}else{
						$wrapper_background  = '<div class="gdlr-core-pbf-background-wrap" ' . gdlr_core_esc_style(array(
							'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
								'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
							))? '': $options['value']['border-radius'],
							'background-shadow-size' => empty($options['value']['background-shadow-size'])? '': $options['value']['background-shadow-size'],
							'background-shadow-color' => empty($options['value']['background-shadow-color'])? '': $options['value']['background-shadow-color'],
							'background-shadow-opacity' => empty($options['value']['background-shadow-opacity'])? '': $options['value']['background-shadow-opacity'],
						)) . ' >';
						$wrapper_background .= '<div class="gdlr-core-pbf-background gdlr-core-parallax gdlr-core-js" ' . gdlr_core_esc_style(array(
							'background-position' => 'center',
							'background-size' => 'cover',
						));
						$wrapper_background .= ' data-parallax-speed="' . (empty($options['value']['parallax-speed'])? 0: $options['value']['parallax-speed']) . '" ';
						$wrapper_background .= ' data-video-fallback="' . (empty($options['value']['background-video-image'])? '': gdlr_core_get_image_url($options['value']['background-video-image'])) . '" ';
						$wrapper_background .= ' >';
						$wrapper_background .= '<div class="gdlr-core-pbf-background-video" data-background-type="video" >';
						
						if( $options['value']['background-type'] == 'video' ){
							if( !empty($options['value']['background-video-url']) ){
								$wrapper_background .= gdlr_core_get_video(
									$options['value']['background-video-url'], 
									array('width' => '100%', 'height' => '100%'), 
									array('background' => 1)
								);
							}
						}else if( $options['value']['background-type'] == 'html5-video' ){
							$wrapper_background .= '<video autoplay loop muted >';
							if( $options['value']['background-video-url-mp4'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-mp4']) . '" type="video/mp4">';
							}
							if( $options['value']['background-video-url-webm'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-webm']) . '" type="video/webm">';
							}
							if( $options['value']['background-video-url-ogg'] ){
								$wrapper_background .= '<source src="' . esc_url($options['value']['background-video-url-ogg']) . '" type="video/ogg">';
							}
							$wrapper_background .= '</video>';
						}
						$wrapper_background .= '</div>';
						$wrapper_background .= '</div>'; // gdlr-cpre-pbc-background
						$wrapper_background .= '</div>'; // gdlr-core-pbf-background-wrap
					}
				} // wrapper-background
				
				// border
				if( !empty($options['value']['border-type']) ){
					if( $options['value']['border-type'] == 'outer-border' ){
						$wrapper_style_atts['border-width'] = empty($options['value']['border-width'])? '': $options['value']['border-width'];
						$wrapper_style_atts['border-color'] = empty($options['value']['border-color'])? '': $options['value']['border-color'];
						$wrapper_style_atts['border-style'] = empty($options['value']['border-style'])? '': $options['value']['border-style'];
						$wrapper_style_atts['border-radius'] = (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
							'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
						))? '': $options['value']['border-radius'];
					}else if( $options['value']['border-type'] == 'inner-border' ){
						$wrapper_background .= '<div class="gdlr-core-pbf-background-frame" ' . gdlr_core_esc_style(array(
							'margin' => (empty($options['value']['border-pre-spaces']))? '0px': $options['value']['border-pre-spaces'],
							'border-width' => empty($options['value']['border-width'])? '': $options['value']['border-width'],
							'border-style' => empty($options['value']['border-style'])? '': $options['value']['border-style'],
							'border-color' => empty($options['value']['border-color'])? '': $options['value']['border-color']
						)) . ' ></div>';
					}
				}

				// marquee effect
				$marquee_background = '';
				if( !empty($options['value']['enable-marquee']) && $options['value']['enable-marquee'] == 'enable' &&
					!empty($options['value']['marquee-text']) ){
					$marquee_atts = array(
						'color' => empty($options['value']['marquee-text-color'])? '': $options['value']['marquee-text-color'],
						'opacity' => empty($options['value']['marquee-opacity'])? '': $options['value']['marquee-opacity'],
						'font-size' => empty($options['value']['marquee-font-size'])? '': $options['value']['marquee-font-size'],
						'font-weight' => empty($options['value']['marquee-font-weight'])? '': $options['value']['marquee-font-weight'],
						'letter-spacing' => empty($options['value']['marquee-font-letter-spacing'])? '': $options['value']['marquee-font-letter-spacing'],
					);
					if( !empty($options['value']['marquee-position']) && !empty($options['value']['marquee-position-offset']) ){
						if( $options['value']['marquee-position'] == 'top' ){
							$marquee_atts['top'] = $options['value']['marquee-position-offset'];
							$marquee_atts['bottom'] = 'auto';
						}else{
							$marquee_atts['bottom'] = $options['value']['marquee-position-offset'];
							$marquee_atts['top'] = 'auto';
						}
						
					}

					$marquee_background  = '<div class="gdlr-core-pbf-wrapper-marquee gdlr-core-marquee gdlr-core-title-font gdlr-core-js" ';
					$marquee_background .= empty($options['value']['marquee-speed'])? '': 'data-speed="' . esc_attr($options['value']['marquee-speed']) . '" ';
					$marquee_background .= empty($options['value']['marquee-direction'])? '': 'data-direction="' . esc_attr($options['value']['marquee-direction']) . '" ';
					$marquee_background .= gdlr_core_esc_style($marquee_atts);
					$marquee_background .= ' >' . gdlr_core_text_filter($options['value']['marquee-text']) . '</div>';
				} 

				// device display class
				$wrapper_display_class = '';
				$wrapper_content_class = '';
				if( !empty($options['value']['hide-this-wrapper-in']) && $options['value']['hide-this-wrapper-in'] != 'none' ){
					$wrapper_display_class .= ' gdlr-core-hide-in-' . $options['value']['hide-this-wrapper-in'];
				}
				if( !empty($options['value']['full-height']) && $options['value']['full-height'] == 'enable' ){
					$wrapper_display_class .= ' gdlr-core-wrapper-full-height gdlr-core-js';

					if( !empty($options['value']['centering-content']) && $options['value']['centering-content'] == 'enable' ){
						$wrapper_display_class .= ' gdlr-core-full-height-center';
						$wrapper_content_class .= ' gdlr-core-full-height-content';
					}
				}
				$wrapper_display_class .= empty($options['value']['class'])? '': ' ' . $options['value']['class'];

				$container_class = '';
				$container_style_atts = array();
				if( empty($options['value']['content-layout']) || $options['value']['content-layout'] == 'boxed' ){
					$container_class .= 'gdlr-core-container';
				}else if( $options['value']['content-layout'] == 'custom' ){
					$container_class .= 'gdlr-core-container-custom';
					if( !empty($options['value']['max-width']) ){
						$container_style_atts['max-width'] = $options['value']['max-width'];
					}
				}else if( $options['value']['content-layout'] == 'full' ){
					gdlr_core_set_container(false);

					if( empty($options['value']['enable-space']) || $options['value']['enable-space'] == 'disable' ){
						$container_class .= 'gdlr-core-pbf-wrapper-full-no-space';
					}else{
						$container_class .= 'gdlr-core-pbf-wrapper-full';
					}
				}

				$container_inner_class = '';
				$container_inner_atts = array();
				if( !empty($options['value']['enable-container-background']) && $options['value']['enable-container-background'] == 'enable' ){
					$container_inner_class = ' gdlr-core-pbf-wrapper-container-inner gdlr-core-item-mglr clearfix';
					$container_inner_atts = array(
						'margin' => (empty($options['value']['container-margin']) || $options['value']['container-margin'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? array(): $options['value']['container-margin'],
						'padding' => (empty($options['value']['container-padding']) || $options['value']['container-padding'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? array(): $options['value']['container-padding'],
						'background-color' => empty($options['value']['container-background-color'])? '': $options['value']['container-background-color'],
						'border-radius' => empty($options['value']['container-border-radius'])? '': $options['value']['container-border-radius'],
						'background-shadow-size' => empty($options['value']['container-shadow-size'])? '': $options['value']['container-shadow-size'],
						'background-shadow-color' => empty($options['value']['container-shadow-color'])? '': $options['value']['container-shadow-color'],
						'background-shadow-opacity' => empty($options['value']['container-shadow-opacity'])? '': $options['value']['container-shadow-opacity'],
						'z-index' => empty($options['value']['container-z-index'])? '': $options['value']['container-z-index'],
					);
					if( !empty($container_inner_atts['margin']['left']) || !empty($container_inner_atts['margin']['right']) ){
						$temp_margin  = 0;
						$temp_margin += empty($container_inner_atts['margin']['left'])? 0: intval($container_inner_atts['margin']['left']);
						$temp_margin += empty($container_inner_atts['margin']['right'])? 0: intval($container_inner_atts['margin']['right']);
						if( $temp_margin == 0 ){
							$container_inner_atts['width'] = '100%';
						}else{
							$container_inner_atts['width'] = 'calc(100% - ' . $temp_margin . 'px)';
						}
					}

					if( !empty($options['value']['container-background-color']) &&
						!empty($options['value']['container-background-gradient']) && $options['value']['container-background-gradient'] != 'none' ){

						if( in_array($options['value']['container-background-gradient'], array('top', 'bottom')) ){
							$min_gradient = empty($options['value']['container-background-gradient-min-opacity'])? 0: $options['value']['container-background-gradient-min-opacity'];

							$container_inner_atts['gradient'] = array(
								array(
									$options['value']['container-background-color'],
									($options['value']['container-background-gradient'] == 'top')? $min_gradient: 1 
								),
								array(
									$options['value']['container-background-color'],
									($options['value']['container-background-gradient'] == 'bottom')? $min_gradient: 1 
								)
							);
						}else if( in_array($options['value']['container-background-gradient'], array('gradient', 'gradient-v')) && !empty($options['value']['container-background-gradient-color']) ){
							$container_inner_atts[$options['value']['container-background-gradient']] = array(
								$options['value']['container-background-color'],
								$options['value']['container-background-gradient-color']
							);
						}
						
					}

					if( (!empty($options['value']['container-margin']['left']) && $options['value']['container-margin']['left'] != '0px') ||
						(!empty($options['value']['container-margin']['right']) && $options['value']['container-margin']['right'] != '0px') ){

						$container_inner_margin = intval($options['value']['container-margin']['left']) + intval($options['value']['container-margin']['right']);
						$container_inner_atts['width'] = 'calc(100% - ' . $container_inner_margin . 'px)';
					}
				}

				// overlay background
				$overlay_background = '';
				if( !empty($options['value']['enable-background-image-overlay']) && $options['value']['enable-background-image-overlay'] == 'enable' ){
					$overlay_background  = '<div class="gdlr-core-wrapper-bg-overlay ';
					$overlay_background .= 'gdlr-core-pos-' . (empty($options['value']['background-image-overlay-section'])? 'left': $options['value']['background-image-overlay-section']) . ' ';
					$overlay_background .= '" ' . gdlr_core_esc_style(array(
						'background-image' => empty($options['value']['background-image-overlay'])? '': $options['value']['background-image-overlay'],
						'margin' => (empty($options['value']['background-image-overlay-margin']) || $options['value']['background-image-overlay-margin'] == array(
							'top'=>'0px', 'right'=>'0px', 'bottom'=>'0px', 'left'=>'0px', 'settings'=>'link'
						))? array(): $options['value']['background-image-overlay-margin'],
					)) . ' ></div>';
				}

				if( !is_admin() && !empty($options['value']['privacy']) ){
					$privacy_content = gdlr_core_get_pb_privacy_box($options['value']['privacy'], 'pb-wrapper');
				}
				if( !empty($privacy_content) ){
					$wrapper  = $privacy_content;
				}else{
					$wrapper  = '<div class="gdlr-core-pbf-wrapper ' . esc_attr($wrapper_display_class) . '" ' . gdlr_core_esc_style($wrapper_style_atts);
					$wrapper .= (empty($options['value']['skin'])? '': 'data-skin="' . esc_attr($options['value']['skin']) . '" ');
					$wrapper .= (empty($options['value']['id'])? '': ' id="' . esc_attr($options['value']['id']) . '" ');
					$wrapper .= (empty($options['value']['decrease-height']) || $options['value']['decrease-height'] == '0px')? '': ' data-decrease-height="' . esc_attr($options['value']['decrease-height']) . '"';
					$wrapper .= (empty($options['value']['float-social-id']))? '': ' data-float-social="' . esc_attr($options['value']['float-social-id']) . '"';
					$wrapper .= '>' . $wrapper_background . $overlay_background . $marquee_background;

					// for background gradient
					if( !empty($options['value']['wrapper-background-gradient']) && in_array($options['value']['wrapper-background-gradient'], array('both', 'top', 'bottom')) ){
						if( in_array($options['value']['wrapper-background-gradient'], array('top', 'both')) ){
							$wrapper .= '<div class="gdlr-core-page-builder-wrapper-top-gradient" ' . gdlr_core_esc_style(array(
								'height' => empty($options['value']['wrapper-background-top-gradient-size'])? 0: $options['value']['wrapper-background-top-gradient-size'],
								'opacity' => empty($options['value']['wrapper-background-gradient-opacity'])? '': $options['value']['wrapper-background-gradient-opacity'],
								'gradient' => array(
									array(
										empty($options['value']['wrapper-background-gradient-color'])? '#000': $options['value']['wrapper-background-gradient-color'], 
										100
									),
									array(
										empty($options['value']['wrapper-background-gradient-color'])? '#000': $options['value']['wrapper-background-gradient-color'],
										0
									)
								),
								'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
									'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
								))? '': $options['value']['border-radius'],
							)) . ' ></div>';
						}
						if( in_array($options['value']['wrapper-background-gradient'], array('bottom', 'both')) ){
							$wrapper .= '<div class="gdlr-core-page-builder-wrapper-bottom-gradient" ' . gdlr_core_esc_style(array(
								'height' => empty($options['value']['wrapper-background-bottom-gradient-size'])? 0: $options['value']['wrapper-background-bottom-gradient-size'],
								'opacity' => empty($options['value']['wrapper-background-gradient-opacity'])? '': $options['value']['wrapper-background-gradient-opacity'],
								'gradient' => array(
									array(
										empty($options['value']['wrapper-background-gradient-color'])? '#000': $options['value']['wrapper-background-gradient-color'], 
										0
									),
									array(
										empty($options['value']['wrapper-background-gradient-color'])? '#000': $options['value']['wrapper-background-gradient-color'],
										100
									)
								),
								'border-radius' => (empty($options['value']['border-radius']) || $options['value']['border-radius'] == array( 
									'top'=>'', 'right'=>'', 'bottom'=>'', 'left'=>'', 'settings'=>'link' 
								))? '': $options['value']['border-radius'],
							)) . ' ></div>';
						}
					}	

					if( !empty($options['value']['enable-background-particle']) && $options['value']['enable-background-particle'] == 'enable' ){
						$wrapper .= '<div class="gdlr-core-js gdlr-core-particle-bg" ></div>';
					}
					$wrapper .= '<div class="gdlr-core-pbf-wrapper-content gdlr-core-js ' . esc_attr($wrapper_content_class) . '" ' . gdlr_core_esc_style($content_style_atts) . ' ';
					$wrapper .= gdlr_core_get_animation_atts(array(
						'animation' => (empty($options['value']['animation'])? '': $options['value']['animation']),
						'location' => (empty($options['value']['animation-location'])? '': $options['value']['animation-location'])
					)) . ' >';
					$wrapper .= '<div class="gdlr-core-pbf-wrapper-container clearfix ' . esc_attr($container_class) . '" ' . gdlr_core_esc_style($container_style_atts) .'>';
					$wrapper .= empty($container_inner_class)? '': '<div class="' . esc_attr($container_inner_class) . '" ' . gdlr_core_esc_style($container_inner_atts). ' >';
					if( !empty($options['value']['container-background-image']) ){
						$wrapper .= '<div class="gdlr-core-page-builder-wrapper-container-bg-image" ' . gdlr_core_esc_style(array(
							'background-image' => $options['value']['container-background-image'],
							'opacity' => empty($options['value']['container-background-image-opacity'])? '': $options['value']['container-background-image-opacity'],
							'border-radius' => empty($options['value']['container-border-radius'])? '': $options['value']['container-border-radius'],
						)) . ' ></div>';
						$wrapper .= '<div class="gdlr-core-page-builder-wrapper-container-content" >';
					}
					if( !empty($options['items']) ){
						$wrapper .= gdlr_core_escape_content(call_user_func($callback, $options['items']));
					}
					if( !empty($options['value']['container-background-image']) ){
						$wrapper .= '</div>';
					}
					$wrapper .= empty($container_inner_class)? '': '</div>'; 
					$wrapper .= '</div>'; // gdlr-core-pbf-wrapper-container
					$wrapper .= '</div>'; // gdlr-core-pbf-wrapper-content
					$wrapper .= '</div>'; // gdlr-core-pbf-wrapper
				}
				gdlr_core_set_container(true);

				return $wrapper;
				
			}
			
		} // gdlr_core_pb_wrapper_background
	} // class_exists	